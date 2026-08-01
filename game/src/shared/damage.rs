use bevy::{ecs::entity::MapEntities, prelude::*, time::Stopwatch};
use lightyear::{
    prediction::registry::PredictionBuilderExt,
    prelude::{AppComponentExt, PredictionRegistrationExt},
};
use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::{
    build::TICKRATE,
    shared::{
        combat::{CombatEntityActive, CombatManager, CombatSystemSet},
        enemies::Enemy,
        players::Player,
        states::{AppState, InGameState},
        stats::components::{
            Armor, CritChance, CritDamage, Evasion, Health, HealthRegen, LifeSteal, Shield,
        },
        upgrades::StatUpgradeKind::Damage,
    },
    utils::CreatedBy,
};

pub const SHIELD_RECOVERY_TIME: f32 = 2.0;
pub const HEALTH_REGEN_TICKRATE: f32 = 0.5;

#[derive(
    Component, Debug, Clone, Reflect, Deref, DerefMut, Default, PartialEq, Serialize, Deserialize,
)]
pub struct HealthBuffer {
    buff: Vec<HealthChangeInstance>,
}

/// This is used to track shield replenishment
#[derive(
    Component, Debug, Clone, Reflect, Deref, DerefMut, Default, PartialEq, Serialize, Deserialize,
)]
pub struct TimeSinceLastDamage(pub Stopwatch);

impl HealthBuffer {
    pub fn push_damage(&mut self, from: Entity, dam: f32, crit_info: Option<(f32, f32)>) {
        let mut inst = HealthChangeInstance {
            source: from,
            kind: HealthChange::Damage,
            initial: dam,
            end: dam,
            crit_chance: -1.0,
            crit_multiplier: 1.0,
            result: None,
        };

        if let Some((cc, cd)) = crit_info {
            inst.crit_chance = cc;
            inst.crit_multiplier = cd;
        }
        self.buff.push(inst);
    }
    pub fn push_heal(&mut self, from: Entity, heal: f32, crit_info: Option<(f32, f32)>) {
        let mut inst = HealthChangeInstance {
            source: from,
            kind: HealthChange::Heal,
            initial: heal,
            end: heal,
            crit_chance: -1.0,
            crit_multiplier: 1.0,
            result: None,
        };

        if let Some((cc, cd)) = crit_info {
            inst.crit_chance = cc;
            inst.crit_multiplier = cd;
        }
        self.buff.push(inst);
    }
}

impl MapEntities for HealthBuffer {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        for inst in &mut self.buff {
            inst.source = entity_mapper.get_mapped(inst.source);
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Reflect, Debug)]
pub enum HealthChange {
    Heal,
    Damage,
}

/// Stores all records of healing and damage that will be dealt to this unit on this frame
/// this is totaled up at the end of combat and then used to calculate all damage results
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Reflect, Debug)]
pub struct HealthChangeInstance {
    pub source: Entity,
    pub kind: HealthChange,
    // Marks the unmitigated amount
    pub initial: f32,
    pub end: f32,
    crit_chance: f32,
    crit_multiplier: f32,
    result: Option<HealthChangeResult>,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Reflect, Debug)]
pub enum HealthChangeResult {
    /// The change was not a crit
    Normal,
    /// This expects to have the new value inside the health change
    Crit,
    /// Entity resisted damage
    Invulnerable,
    /// Evaded
    Evaded,
    /// This gets written when something was Apply, but its overkill damage
    EntityAlreadyDead,
    /// This can be the result when, for example, we're healing someone at full health
    DidNothing,
}

/// Records when an entity's shield value has gone to 0 as a result of damage
#[derive(EntityEvent)]
pub struct ShieldBroken {
    pub entity: Entity,
}

/// A message that gets written which displays the result of damage that was dealt by entities.
/// The idea here is that we can buffer all of these and allow other systems to react to these interactions.
/// For example, we could only show players damage which is dealt by them or by projectiles that they
/// create
#[derive(Message, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HealthChangeMessage {
    pub source_entity: Entity,
    pub receiving_entity: Entity,
    pub kind: HealthChange,
    pub result: HealthChangeResult,
    pub amount: f32,
}

/// This marks that an entity has had their health reduced to below 0.
/// Entities who are dead may not necessarily end up fully dead. They may
/// instead end up getting revived. This component gets added to things
/// to let us know "hey, one of those things is happening", which helps avoid targeting
/// dead OR reviving OR dying entities
#[derive(Component, Clone, Serialize, Deserialize, Reflect, Debug)]
pub enum DeathState {
    Dying(Timer),
    Reviving(Timer),
    Dead,
}

pub struct SharedDamagePlugin;

impl Plugin for SharedDamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<HealthChangeMessage>()
            .add_message::<EntityKilledMessage>()
            .add_systems(
                FixedPostUpdate,
                ((
                    apply_health_regen,
                    check_invulnerability_conditions,
                    check_evasion,
                    roll_critical,
                    apply_damage_mitigation_to_incoming_damage,
                    register_damage_to_apply,
                    apply_frame_changes,
                    clear_damage_buffer,
                    apply_lifesteal,
                    tick_damage_timer,
                    recharge_shield,
                )
                    .chain()
                    .in_set(CombatSystemSet::Cleanup)
                    .run_if(in_state(InGameState::InGame)),),
            );
    }
}

pub struct DamageProtocolPlugin;
impl Plugin for DamageProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.component::<HealthBuffer>().predict();
    }
}

#[derive(Message)]
pub struct EntityKilledMessage {
    pub dead_entity: Entity,
    pub responsible_entity: Entity,
}

/// We only push health regen when the entity is below max health, or somewhat eagerly when
/// the buffer has other stuff in it. This may end up doing nothing some of the time, but we handle
/// that downstream
fn apply_health_regen(
    time: Res<Time<Virtual>>,
    mut timer: Local<Option<Timer>>,
    mut q_heal: Query<(Entity, &mut HealthBuffer, &Health, &HealthRegen)>,
) {
    if timer.is_none() {
        *timer = Some(Timer::from_seconds(
            HEALTH_REGEN_TICKRATE,
            TimerMode::Repeating,
        ));
    }
    if let Some(ref mut t) = *timer {
        t.tick(time.delta());
        if t.just_finished() {
            for (ent, mut buff, health, regen) in &mut q_heal {
                if (health.current < health.max()) || (!buff.buff.is_empty()) {
                    let to_heal = regen.0 * (HEALTH_REGEN_TICKRATE / 5.0);
                    buff.push_heal(ent, to_heal, None)
                }
            }
        }
    }
}

/// Allows us to short-circuit this process, so to speak, because we know that the result of all damage for this
/// entity is going to be "yeah you can't do that"
///
/// As of right now, I don't have any invulnerable conditions, so this is a no op
fn check_invulnerability_conditions(q_damage: Query<&HealthBuffer>) {}

/// Evades all damage in the frame
fn check_evasion(
    mut combat: ResMut<CombatManager>,
    mut q_health: Query<(&Evasion, &mut HealthBuffer)>,
) {
    for (evasion, mut health) in &mut q_health {
        let roll = combat.rng.random_range(0.0..1.0);
        if roll <= evasion.0 {
            for change in &mut health.buff {
                match change.kind {
                    HealthChange::Damage => {
                        change.result = Some(HealthChangeResult::Evaded);
                        change.end = 0.0;
                    }
                    _ => {}
                }
            }
        }
    }
}

/// This will roll a crit if you're not invuln and haven't evaded
fn roll_critical(mut combat_manager: ResMut<CombatManager>, mut q_buff: Query<&mut HealthBuffer>) {
    for mut health in (&mut q_buff) {
        for mut change in &mut health.buff {
            if change.result.is_some() {
                continue;
            }
            let roll = combat_manager.rng.random_range(0.0..1.0);
            let is_crit = roll <= change.crit_chance;
            trace!("Roll: {:?}, Crit Chance: {:?}", roll, change.crit_chance);
            if is_crit {
                change.end = change.end * change.crit_multiplier;
                trace!("This is a crit");
                change.result = Some(HealthChangeResult::Crit);
            }
        }
    }
}

/// This must read and modify the end value because crits have already been rolled and written
/// to end
fn apply_damage_mitigation_to_incoming_damage(
    mut q_incoming: Query<(&mut HealthBuffer, Option<&Armor>)>,
) {
    for (mut buffer, m_armor) in &mut q_incoming {
        for mut instance in &mut buffer.buff {
            let res = if let Some(a) = m_armor {
                a.mitigate_incoming_damage(instance.end)
            } else {
                instance.end
            };
            instance.end = res
        }
    }
}

/// Run after checking invulnerability, evasion, crit, and reducing the output, we're finally ready to
/// pick up the stragglers
fn register_damage_to_apply(mut q_health: Query<&mut HealthBuffer>) {
    for mut health in &mut q_health {
        for change in &mut health.buff {
            if (change.end >= 0.0 && change.result.is_none()) {
                change.result = Some(HealthChangeResult::Normal)
            }
        }
    }
}

fn apply_frame_changes(
    mut commands: Commands,
    mut events: MessageWriter<EntityKilledMessage>,
    //mut damage_events: MessageWriter<AppliedDamageLogMessage>,
    mut q_health: Query<
        (Entity, &mut HealthBuffer, &mut Health, Option<&mut Shield>),
        CombatEntityActive,
    >,
) {
    for (ent, mut health, mut hp, mut m_shield) in &mut q_health {
        let mut dead = false;
        let mut killed_by = None;
        let has_shield = m_shield.is_some();
        let mut shield_broken = false;
        for mut change in &mut health.buff {
            let mult = match change.kind {
                HealthChange::Heal => 1.0,
                HealthChange::Damage => -1.0,
            };

            let to_apply = match change.result.unwrap() {
                HealthChangeResult::Normal | HealthChangeResult::Crit => {
                    let res = change.end * mult;
                    trace!("Amount: {:?}", res);
                    res
                }
                _ => {
                    trace!("Applying 0 damage");
                    0.0
                }
            };

            if to_apply > 0.0 {
                if hp.current == hp.max() {
                    change.result = Some(HealthChangeResult::DidNothing)
                } else {
                    hp.current = (hp.current + to_apply).clamp(-1.0, hp.max());
                    trace!("Changed health to {:?}", hp.current);
                }
            } else if to_apply < 0.0 {
                let health_to_sub = if has_shield && !shield_broken {
                    let mut s = m_shield.as_mut().unwrap();
                    trace!("Starting shield value {:?}", s.current);
                    s.current += to_apply;
                    if s.current <= 0.0 {
                        shield_broken = true;
                        trace!("Ending shield value {:?}", s.current);
                        (0.0 + s.current)
                    } else {
                        trace!("Ending shield value {:?}", s.current);
                        0.0
                    }
                } else {
                    to_apply
                };

                hp.current = (hp.current + health_to_sub).clamp(-1.0, hp.max());
                trace!("Changed health to {:?}", hp.current);
                if hp.current <= 0.0 && !dead {
                    killed_by = Some(change.source);
                    dead = true;
                }
            }

            trace!("Got result: {:?}", change.result);
        }
        if shield_broken {
            commands.trigger(ShieldBroken { entity: ent })
        }
        if dead {
            events.write(EntityKilledMessage {
                dead_entity: ent,
                responsible_entity: killed_by.unwrap(),
            });
        }
    }
}

fn recharge_shield(mut q_shield: Query<(&mut Shield, &TimeSinceLastDamage)>) {
    for (mut shield, time) in &mut q_shield {
        if shield.current == shield.max() {
            continue;
        }
        if time.elapsed_secs() >= SHIELD_RECOVERY_TIME {
            let recharge_rate = (3.0 * (time.elapsed_secs() - SHIELD_RECOVERY_TIME).powf(2.0));
            shield.current =
                (shield.current + (recharge_rate * shield.max())).clamp(0.0, shield.max())
        }
    }
}

/// Since we're clearing the buffer here, we're going to write the damage logging events for
/// other systems to look at
fn clear_damage_buffer(
    mut commands: Commands,
    mut messages: MessageWriter<HealthChangeMessage>,
    mut q_buffer: Query<(Entity, &mut HealthBuffer)>,
    mut q_not_damaged: Query<&mut TimeSinceLastDamage>,
) {
    for (ent, mut buff) in &mut q_buffer {
        if !buff.is_empty() {
            if let Ok(mut dt) = q_not_damaged.get_mut(ent) {
                dt.reset();
            } else {
                warn!("Entity with a damage buffer and with no TimeSinceLastDamage")
            }
        }
        for mut inst in buff.drain(..) {
            let result = inst.result.take();
            messages.write(HealthChangeMessage {
                source_entity: inst.source,
                kind: inst.kind,
                receiving_entity: ent,
                result: result.unwrap(),
                amount: inst.end,
            });
        }
    }
}

fn tick_damage_timer(time: Res<Time<Virtual>>, q_timer: Query<&mut TimeSinceLastDamage>) {
    for mut timer in q_timer {
        (*timer).tick(time.delta());
    }
}

/// This has to read the messages of successful damage, or else it will
/// be cleared in the buffer before its applied
///
/// Players and enemies get their LS downscaled because they need to hold
/// base values of 1 for their weapons and attacks to scale properly.
/// This could probably be better formalized later
fn apply_lifesteal(
    mut messages: MessageReader<HealthChangeMessage>,
    mut q_health: Query<(&mut HealthBuffer, &LifeSteal, Has<Player>, Has<Enemy>)>,
    q_creator: Query<(&CreatedBy, &LifeSteal)>,
) {
    for change in messages.read() {
        match (change.kind, change.result) {
            (HealthChange::Damage, (HealthChangeResult::Crit | HealthChangeResult::Normal)) => {
                if let Ok((mut buff, ls, is_player, is_enemy)) =
                    q_health.get_mut(change.source_entity)
                {
                    let ls_multiplier = if !is_player && !is_enemy {
                        ls.0
                    } else {
                        ls.0 - 1.0
                    };
                    let amount_to_apply = change.amount * ls_multiplier;
                    if amount_to_apply > 0.0 {
                        buff.push_heal(change.source_entity, amount_to_apply, None)
                    }
                }

                if let Ok((cb, source_ls)) = q_creator.get(change.source_entity) {
                    if let Ok((mut buff, _ls, _is_player, _is_enemy)) = q_health.get_mut(cb.0) {
                        let amount_to_apply = change.amount * source_ls.0;
                        buff.push_heal(change.source_entity, amount_to_apply, None)
                    }
                }
            }
            _ => {}
        }
    }
}
