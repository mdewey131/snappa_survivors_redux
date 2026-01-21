use super::{AppliesCollisionEffect, CollisionEffect};
use avian2d::prelude::*;
use bevy::prelude::*;
/// Generic system for applying collision effects on the trigger of a collision start
pub fn apply_collision_effect_on_collision_start<E: CollisionEffect>(
    mut commands: Commands,
    mut col_events: MessageReader<CollisionStart>,
    q_trigger: Query<&AppliesCollisionEffect<E>>,
    q_collided_with: Query<&CollisionLayers>,
) {
    for col in col_events.read() {
        // lesson learned here after testing - there can arise situations where an entity with an effect is
        // colliding with another entity that also has that effect. If this is not handled properly, you will
        // end up missing collisions that should go through because it doesn't seem like the other entity should
        // fire. For example, projeciles colliding with enemies. So, we have to test both possibilities and then
        // fire each one if it's valid.
        // To fire should always be loaded in the order (triggering entity, triggered upon)
        let mut to_fire = Vec::new();
        if let Ok(t) = q_trigger.get(col.collider1) {
            if let Ok(layers) = q_collided_with.get(col.collider2) {
                if (layers.memberships.0 & t.to.0) != 0 {
                    to_fire.push((t, col.collider1, col.collider2))
                }
            }
        }

        // Test the other side
        if let Ok(t) = q_trigger.get(col.collider2) {
            if let Ok(layers) = q_collided_with.get(col.collider1) {
                if (layers.memberships.0 & t.to.0) != 0 {
                    to_fire.push((t, col.collider2, col.collider1))
                }
            }
        }

        for (trigger, applies_from, applies_to) in &to_fire {
            trigger
                .eff
                .apply_to(&mut commands, *applies_to, *applies_from);
        }
    }
}
