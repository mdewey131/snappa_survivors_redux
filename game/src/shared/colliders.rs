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
        let (mems, filters) = if predicted {
            (
                [ColliderTypes::PredictedEnemy],
                [
                    ColliderTypes::PredictedEnemy,
                    ColliderTypes::PredictedPlayer,
                ],
            )
        } else {
            (
                [ColliderTypes::ReplicatedEnemy],
                [
                    ColliderTypes::ReplicatedEnemy,
                    ColliderTypes::ReplicatedPlayer,
                ],
            )
        };

        Self::new(
            RigidBody::Dynamic,
            Collider::capsule(20.0, 30.0),
            1.0,
            mems.into(),
            filters.into(),
        )
    }

    pub fn player(predicted: bool) -> Self {
        let (mems, filters) = if predicted {
            (
                [ColliderTypes::PredictedPlayer],
                [
                    ColliderTypes::PredictedEnemy,
                    ColliderTypes::PredictedStaticPickup,
                    ColliderTypes::PredictedRemotePickup,
                ],
            )
        } else {
            (
                [ColliderTypes::ReplicatedPlayer],
                [
                    ColliderTypes::ReplicatedEnemy,
                    ColliderTypes::ReplicatedStaticPickup,
                    ColliderTypes::ReplicatedRemotePickup,
                ],
            )
        };

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
///
/// Becuase this has to be a "flat" enum, and because
/// the single player mode exists, this has to make a
/// distinction between predicted and replicated
/// types in each level. I hate it, but I don't know
/// what else to do about it
#[derive(PhysicsLayer, Default, Clone, Copy, Debug)]
pub enum ColliderTypes {
    #[default]
    PredictedPlayer,
    ReplicatedPlayer,
    PredictedEnemy,
    ReplicatedEnemy,
    PredictedPlayerProjectile,
    ReplicatedPlayerProjectile,
    PredictedEnemyProjectile,
    ReplicatedEnemyProjectile,
    PredictedPlayerPickupRadius,
    ReplicatedPlayerPickupRadius,
    /// Has to be run over by the player
    PredictedStaticPickup,
    ReplicatedStaticPickup,
    //Can be picked up by pickup radius
    PredictedRemotePickup,
    ReplicatedRemotePickup,
    PredictedPlayerRevive,
    ReplicatedPlayerRevive,
}
