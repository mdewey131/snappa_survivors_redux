use std::marker::PhantomData;

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
    pub fn enemy(predicted: bool) -> Self {
        let networking_col_type = if predicted {
            ColliderTypes::Predicted
        } else {
            ColliderTypes::Replicated
        };
        let mems = [ColliderTypes::Enemy, networking_col_type];
        let filters = [
            ColliderTypes::Player,
            ColliderTypes::Enemy,
            networking_col_type,
        ];
        Self::new(
            RigidBody::Dynamic,
            Collider::capsule(20.0, 30.0),
            1.0,
            mems.into(),
            filters.into(),
        )
    }

    pub fn player(predicted: bool) -> Self {
        let networking_col_type = if predicted {
            ColliderTypes::Predicted
        } else {
            ColliderTypes::Replicated
        };
        let mems = [ColliderTypes::Player, networking_col_type];
        let filters = [
            ColliderTypes::Enemy,
            networking_col_type,
            ColliderTypes::StaticPickup,
            ColliderTypes::RemotePickup,
        ];
        Self::new(
            RigidBody::Dynamic,
            Collider::capsule(20.0, 30.0),
            1.0,
            mems.into(),
            filters.into(),
        )
    }

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
    // These two added layers will be used to distinguish between
    // different networking types, because in single player mode,
    // we want to make sure that a replicated enemy won't block
    // a predicted player
    Replicated,
    Predicted,
}
