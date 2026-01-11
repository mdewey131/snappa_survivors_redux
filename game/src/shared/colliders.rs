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
#[derive(PhysicsLayer, Default)]
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
