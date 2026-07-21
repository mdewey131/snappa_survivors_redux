use bevy::{ecs::entity::MapEntities, prelude::*};
use lightyear::{
    prediction::registry::PredictionBuilderExt,
    prelude::{AppComponentExt, PredictionRegistrationExt},
};
use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::shared::{
    combat::{CombatEntityActive, CombatManager, CombatSystemSet},
    stats::components::{Armor, CritChance, CritDamage, Health},
};

#[derive(
    Component, Debug, Clone, Reflect, Deref, DerefMut, Default, PartialEq, Serialize, Deserialize,
)]
pub struct DamageBuffer {
    buff: Vec<DamageInstance>,
}

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
    /// There's a lot of reasons this can happen, but they generally
    /// all point to "this entity is invulnerable right now for some reason"
    DamageNegated,
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
                    roll_critical,
                    apply_damage_mitigation_to_incoming_damage,
                    apply_frame_damage,
                    clear_damage_buffer,
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

/// Allows us to short-circuit this process, so to speak, because we know that the result of all damage for this
/// entity is going to be "yeah you can't do that"
fn check_invulnerability_conditions() {}

/// For
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
            // Assuming at this point that it's going to be successful. Check this later
            instance.result = Some(DamageResult::Apply(outgoing_damage))
        }
    }
}

fn apply_frame_damage(
    mut events: MessageWriter<EntityKilledMessage>,
    //mut damage_events: MessageWriter<AppliedDamageLogMessage>,
    mut q_health: Query<(Entity, &mut DamageBuffer, &mut Health), CombatEntityActive>,
) {
    for (ent, mut buff, mut health) in &mut q_health {
        let mut health_to_set = health.current;
        let mut dead = false;
        let mut killed_by = None;
        let _total_damage = buff
            .iter_mut()
            .map(|dam| {
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

                health_to_set -= to_apply;
                /*
                damage_events.write(AppliedDamageLogMessage {
                    source: dam.damage_source,
                    amount: dam.amount,
                });
                */
                if health_to_set <= 0.0 && !dead {
                    killed_by = Some(dam.damage_source);
                    dead = true;
                }
                to_apply
            })
            .sum::<f32>();
        health.current = health_to_set.clamp(0.0, health.max());
        if dead {
            events.write(EntityKilledMessage {
                dead_entity: ent,
                responsible_entity: killed_by.unwrap(),
            });
        }
    }
}

/// Since we're clearing the buffer here, we're going to write the damage logging events for
/// other systems to look at
fn clear_damage_buffer(
    mut messages: MessageWriter<DamageResultMessage>,
    mut q_buffer: Query<(Entity, &mut DamageBuffer)>,
) {
    for (ent, mut buff) in &mut q_buffer {
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
