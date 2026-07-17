use bevy::{ecs::entity::MapEntities, prelude::*};
use lightyear::{
    prediction::registry::PredictionBuilderExt,
    prelude::{AppComponentExt, PredictionRegistrationExt},
};
use serde::{Deserialize, Serialize};

use crate::shared::{
    combat::{CombatEntityActive, CombatSystemSet},
    stats::components::Health,
};

#[derive(
    Component, Debug, Clone, Reflect, Deref, DerefMut, Default, PartialEq, Serialize, Deserialize,
)]
pub struct DamageBuffer {
    buff: Vec<DamageInstance>,
}

impl DamageBuffer {
    pub fn push_damage(&mut self, from: Entity, dam: f32, crit_chance: Option<f32>) {
        let mut dam = DamageInstance {
            damage_source: from,
            amount: dam,
            crit_chance: 0.0,
            result: None,
        };

        if let Some(cc) = crit_chance {
            dam.crit_chance = cc
        }

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
    /// Expects a number in [0.0, 1.0]
    crit_chance: f32,
    result: Option<DamageResult>,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Reflect, Debug)]
pub enum DamageResult {
    /// The damage is going to go through, this is the amount to apply
    Apply(f32),
    /// There's a lot of reasons this can happen, but they generally
    /// all point to "this entity is invulnerable right now for some reason"
    DamageNegated,
}

/// Records when an entity's shield value has gone to 0 as a result of damage
#[derive(EntityEvent)]
pub struct ShieldBroken {
    pub entity: Entity,
}

/// Sent to player entities responsible for this damage to tell them the net damage dealt
pub struct DamageMessage {
    pub dealt_by: Entity,
    pub damage: f32,
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
        app.add_message::<EntityKilledMessage>().add_systems(
            FixedPostUpdate,
            ((apply_frame_damage, clear_damage_buffer)
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

fn apply_frame_damage(
    mut events: MessageWriter<EntityKilledMessage>,
    //mut damage_events: MessageWriter<AppliedDamageLogMessage>,
    mut q_health: Query<(Entity, &DamageBuffer, &mut Health), CombatEntityActive>,
) {
    for (ent, buff, mut health) in &mut q_health {
        let mut health_to_set = health.current;
        let mut dead = false;
        let mut killed_by = None;
        let _total_damage = buff
            .iter()
            .map(|dam| {
                health_to_set -= dam.amount;
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
                dam.amount
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

fn clear_damage_buffer(mut q_buffer: Query<&mut DamageBuffer>) {
    for mut buff in &mut q_buffer {
        buff.drain(..);
    }
}
