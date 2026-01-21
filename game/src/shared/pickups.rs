use crate::shared::{colliders::*, combat::CombatSystemSet, states::InGameState};
use avian2d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct XPPickup {
    pub val: f32,
    pub targeting: Option<Entity>,
}
impl XPPickup {
    pub fn new(v: f32) -> Self {
        Self {
            val: v,
            targeting: None,
        }
    }
}

/// Marks the collision effect where the orb follows the player upon contact with their collision layer
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct XPPickupFollowPlayer;
impl CollisionEffect for XPPickupFollowPlayer {
    fn apply_to(&self, coms: &mut Commands, to: Entity, from: Entity) {
        coms.queue(move |world: &mut World| {
            // The colliding entity in this case is the child of the player, so we have to do this
            let parent = world.get::<ChildOf>(to);
            let p_ent = if let Some(p) = parent {
                p.0
            } else {
                return;
            };
            let mut ent_mut = world.get_entity_mut(from);
            if let Ok(ref mut em) = ent_mut {
                unsafe {
                    let xp_orb = em.get_components_mut_unchecked::<&mut XPPickup>();
                    if let Some(mut xp) = xp_orb {
                        xp.targeting = Some(p_ent)
                    }
                }
                em
                    // We don't want the ability to follow once we have started following
                    // This initially removed the collider. Don't do that, you won't be able to pick up the orb after
                    .remove::<AppliesCollisionEffect<XPPickupFollowPlayer>>();
            }
        });
    }
}

pub struct SharedPickupsPlugin;
impl Plugin for SharedPickupsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (xp_orb_update)
                .in_set(CombatSystemSet::Combat)
                .run_if(in_state(InGameState::InGame)),
        )
        .add_systems(
            FixedPostUpdate,
            (
                apply_collision_effect_on_collision_start::<XPPickupFollowPlayer>,
                apply_collision_effect_on_collision_start::<TriggerPickup>,
            )
                .after(PhysicsSystems::Last)
                .in_set(CombatSystemSet::PostPhysicsSet)
                .run_if(in_state(InGameState::InGame)),
        )
        .add_observer(add_xp_collider_components);
    }
}

fn xp_orb_update(
    mut q_position: Query<(&Position, &mut LinearVelocity, &XPPickup)>,
    q_player: Query<&Position, Without<XPPickup>>,
) {
    for (xp_pos, mut xp_lv, pickup) in &mut q_position {
        if let Some(t_ent) = pickup.targeting {
            if let Ok(t_pos) = q_player.get(t_ent) {
                xp_lv.0 = (t_pos.0 - xp_pos.0).normalize_or_zero() * 50.0;
            }
        }
    }
}

fn add_xp_collider_components(trig: On<Add, XPPickup>, mut commands: Commands) {
    commands.entity(trig.entity).insert((
        CommonColliderBundle::new(
            RigidBody::Kinematic,
            Collider::circle(10.0),
            1.0,
            [ColliderTypes::RemotePickup].into(),
            [ColliderTypes::Player, ColliderTypes::PlayerPickupRadius].into(),
        ),
        Sensor,
        AppliesCollisionEffect::new(
            [ColliderTypes::PlayerPickupRadius].into(),
            XPPickupFollowPlayer,
        ),
        AppliesCollisionEffect::new([ColliderTypes::Player].into(), TriggerPickup),
    ));
}

#[derive(EntityEvent)]
pub struct PickupTrigger {
    entity: Entity,
    /// Whatever the entity is that ran into the pickup
    pub apply_to: Entity,
}

impl PickupTrigger {
    pub fn new(from: Entity, to: Entity) -> Self {
        Self {
            entity: from,
            apply_to: to,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Reflect)]
pub struct TriggerPickup;
impl CollisionEffect for TriggerPickup {
    fn apply_to(&self, coms: &mut Commands, to: Entity, from: Entity) {
        coms.trigger(PickupTrigger::new(from, to));
    }
}
