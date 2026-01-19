use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shared::{combat::CombatSystemSet, stats::components::Health};

#[derive(Component, Debug, Clone, Reflect, Deref, DerefMut, Default)]
pub struct DamageBuffer(Vec<DamageInstance>);

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Reflect, Debug)]
pub struct DamageInstance {
    pub damage_source: Entity,
    pub amount: f32,
}

#[derive(Component, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Reflect, Debug)]
pub struct Dead;

pub struct SharedDamagePlugin;

impl Plugin for SharedDamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EntityKilledMessage>().add_systems(
            FixedPostUpdate,
            ((
                apply_frame_damage,
                clear_damage_buffer,
                apply_dead_component,
            )
                .chain()
                .in_set(CombatSystemSet::Cleanup),),
        );
    }
}

#[derive(Message)]
pub struct EntityKilledMessage {
    dead_entity: Entity,
    responsible_entity: Entity,
}

fn apply_frame_damage(
    mut events: MessageWriter<EntityKilledMessage>,
    //mut damage_events: MessageWriter<AppliedDamageLogMessage>,
    mut q_health: Query<(Entity, &DamageBuffer, &mut Health), Without<Dead>>,
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
        health.current = health_to_set.clamp(0.0, health.max);
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

/// This uses q_check because if you try to insert this on a predicted component and the confirmed entity is dead on the same frame,
/// then you will end up adding the same component twice in a frame
fn apply_dead_component(
    mut commands: Commands,
    mut events: MessageReader<EntityKilledMessage>,
    //q_check: Query<(), Or<(With<Interpolated>, With<Predicted>)>>,
) {
    for e in events.read() {
        /*if q_check.get(e.dead_entity).is_err()  { */
        commands.entity(e.dead_entity).insert(Dead);
        //}
    }
}
