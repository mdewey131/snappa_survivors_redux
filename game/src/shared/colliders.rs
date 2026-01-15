use avian2d::prelude::*;
use bevy::prelude::*;
/// The common set of things that every entity with a RigidBody in this game must have
#[derive(Bundle)]
pub struct CommonColliderBundle {
    r: RigidBody,
    c: Collider,
    l: LockedAxes,
    m: Mass,
    layers: CollisionLayers,
    en: CollisionEventsEnabled,
}

impl CommonColliderBundle {
    pub fn new(
        r: RigidBody,
        c: Collider,
        mass: f32,
        layer_membership: LayerMask,
        layer_filter: LayerMask,
    ) -> Self {
        Self {
            r,
            c,
            l: LockedAxes::ROTATION_LOCKED,
            m: Mass(mass),
            layers: CollisionLayers::new(layer_membership, layer_filter),
            en: CollisionEventsEnabled,
        }
    }
}

/// Each collider that an entity takes will be a child
/// each of those children will have specified collider sizes
/// and individual filters
///
/// Becuase this has to be a "flat" enum, and because
/// the single player mode exists, this has to make a
/// distinction between predicted and replicated
/// types in each level. I hate it, but I don't know
/// what else to do about it
#[derive(PhysicsLayer, Default, Clone, Copy, Debug)]
pub enum ColliderTypes {
    #[default]
    Player,
    Enemy,
    PlayerProjectile,
    EnemyProjectile,
    PlayerPickupRadius,
    /// Has to be run over by the player
    StaticPickup,
    //Can be picked up by pickup radius
    RemotePickup,
    PlayerRevive,
}
