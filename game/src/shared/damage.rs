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
        stats::components::{Armor, CritChance, CritDamage, Evasion, Health, HealthRegen, Shield},
    },
};

pub const SHIELD_RECOVERY_TIME: f32 = 2.0;

#[derive(
    Component, Debug, Clone, Reflect, Deref, DerefMut, Default, PartialEq, Serialize, Deserialize,
)]
pub struct DamageBuffer {
    buff: Vec<DamageInstance>,
}
#[derive(
    Component, Debug, Clone, Reflect, Deref, DerefMut, Default, PartialEq, Serialize, Deserialize,
)]
pub struct TimeSinceLastDamage(pub Stopwatch);

impl DamageBuffer {
    pub fn push_damage(&mut self, from: Entity, dam: f32) {
        let mut dam = DamageInstance {
            damage_source: from,
            amount: dam,
            crit: false,
            result: None,
        };

        self.buff.push(dam);
    }
}

impl MapEntities for DamageBuffer {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        for inst in &mut self.buff {
            inst.damage_source = entity_mapper.get_mapped(inst.damage_source);
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Reflect, Debug)]
pub struct DamageInstance {
    pub damage_source: Entity,
    // Marks the UNMITIGATED damage
    pub amount: f32,
    crit: bool,
    result: Option<DamageResult>,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Reflect, Debug)]
pub enum DamageResult {
    /// The damage is going to go through, this is the amount to apply
    Apply(f32),
    Invulnerable,
    Evaded,
    /// This gets written when something was Apply, but its overkill damage
    EntityAlreadyDead,
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
pub struct DamageResultMessage {
    pub damaging_entity: Entity,
    pub damaged_entity: Entity,
    pub crit: bool,
    pub result: DamageResult,
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
        app.add_message::<DamageResultMessage>()
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
                    apply_frame_damage,
                    clear_damage_buffer,
                    tick_damage_timer,
                    recharge_shield,
                )
                    .chain()
                    .in_set(CombatSystemSet::Cleanup),),
            );
    }
}

pub struct DamageProtocolPlugin;
impl Plugin for DamageProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.component::<DamageBuffer>().predict();
    }
}

#[derive(Message)]
pub struct EntityKilledMessage {
    pub dead_entity: Entity,
    pub responsible_entity: Entity,
}

fn apply_health_regen(mut q_heal: Query<(&mut Health, &HealthRegen)>) {
    for (mut hp, regen) in &mut q_heal {
        let to_heal = (TICKRATE as f32) * (5.0 / regen.0);
        hp.current = (hp.current + to_heal).clamp(0.0, hp.max())
    }
}

/// Allows us to short-circuit this process, so to speak, because we know that the result of all damage for this
/// entity is going to be "yeah you can't do that"
///
/// As of right now, I don't have any invulnerable conditions, so this is a no op
fn check_invulnerability_conditions(q_damage: Query<&DamageBuffer>) {}

fn roll_critical(
    mut combat_manager: ResMut<CombatManager>,
    mut q_buff: Query<&mut DamageBuffer>,
    q_crit_info: Query<(&CritChance, &CritDamage)>,
) {
    for mut damage in (&mut q_buff) {
        for mut dam in &mut damage.buff {
            if let Ok((cc, cd)) = q_crit_info.get(dam.damage_source) {
                let roll = combat_manager.rng.random_range(0.0..1.0);
                let is_crit = roll <= cc.0;
                info!("Roll: {:?}, Crit Chance: {:?}", roll, cc);
                if is_crit {
                    info!("This is a crit");
                    dam.amount *= cd.0;
                    dam.crit = true;
                }
            }
        }
    }
}

fn apply_damage_mitigation_to_incoming_damage(
    mut q_incoming: Query<(&mut DamageBuffer, Option<&Armor>)>,
) {
    for (mut buffer, m_armor) in &mut q_incoming {
        for mut instance in &mut buffer.buff {
            let outgoing_damage = if let Some(a) = m_armor {
                a.mitigate_incoming_damage(instance.amount)
            } else {
                instance.amount
            };
            instance.amount = outgoing_damage;
        }
    }
}

fn check_evasion(
    mut combat: ResMut<CombatManager>,
    mut q_damage: Query<(&Evasion, &mut DamageBuffer)>,
) {
    for (evasion, mut damage) in &mut q_damage {
        for dam in &mut damage.buff {
            let roll = combat.rng.random_range(0.0..1.0);
            if roll <= evasion.0 {
                dam.result = Some(DamageResult::Evaded)
            }
        }
    }
}

/// Run after checking invulnerability, evasion, and reducing with armor
fn register_damage_to_apply(mut q_damage: Query<&mut DamageBuffer>) {
    for mut damage in &mut q_damage {
        for dam in &mut damage.buff {
            if (dam.amount >= 0.0 && dam.result.is_none()) {
                dam.result = Some(DamageResult::Apply(dam.amount))
            }
        }
    }
}

fn apply_frame_damage(
    mut events: MessageWriter<EntityKilledMessage>,
    //mut damage_events: MessageWriter<AppliedDamageLogMessage>,
    mut q_health: Query<
        (Entity, &mut DamageBuffer, &mut Health, Option<&mut Shield>),
        CombatEntityActive,
    >,
) {
    for (ent, mut damage, mut health, mut m_shield) in &mut q_health {
        let mut dead = false;
        let mut killed_by = None;
        let has_shield = m_shield.is_some();
        let mut shield_broken = false;
        for mut dam in &mut damage.buff {
            let to_apply = match dam.result.unwrap() {
                DamageResult::Apply(f) => {
                    if dead {
                        dam.result = Some(DamageResult::EntityAlreadyDead);
                        0.0
                    } else {
                        f
                    }
                }
                _ => 0.0,
            };

            let health_to_sub = if has_shield && !shield_broken {
                let mut s = m_shield.as_mut().unwrap();
                info!("Starting shield value {:?}", s.current);
                s.current -= to_apply;
                if s.current <= 0.0 {
                    shield_broken = true;
                    info!("Ending shield value {:?}", s.current);
                    (0.0 - s.current)
                } else {
                    info!("Ending shield value {:?}", s.current);
                    0.0
                }
            } else {
                to_apply
            };

            health.current = (health.current - health_to_sub).clamp(-1.0, health.max());
            info!("Setting health value to be {:?}", health.current);
            if health.current <= 0.0 && !dead {
                killed_by = Some(dam.damage_source);
                dead = true;
            }
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
    mut messages: MessageWriter<DamageResultMessage>,
    mut q_buffer: Query<(Entity, &mut DamageBuffer)>,
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
        for mut dam in buff.drain(..) {
            let result = dam.result.take();
            messages.write(DamageResultMessage {
                damaging_entity: dam.damage_source,
                damaged_entity: ent,
                crit: dam.crit,
                result: result.unwrap(),
            });
        }
    }
}

fn tick_damage_timer(time: Res<Time<Virtual>>, q_timer: Query<&mut TimeSinceLastDamage>) {
    for mut timer in q_timer {
        (*timer).tick(time.delta());
    }
}
