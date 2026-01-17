use avian2d::prelude::*;
use bevy::{ecs::query::QueryFilter, prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shared::{
    colliders::{ColliderTypes, CommonColliderBundle},
    game_kinds::MultiPlayerComponentOptions,
};

pub struct ProjectileProtocolPlugin;
impl Plugin for ProjectileProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Projectile>().add_prediction();
    }
}
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Projectile {
    pub movement: ProjectileMovement,
}

impl From<Projectile> for CommonColliderBundle {
    fn from(value: Projectile) -> Self {
        Self::new(
            RigidBody::Dynamic,
            Collider::rectangle(20.0, 20.0),
            1.0,
            [ColliderTypes::PlayerProjectile].into(),
            [ColliderTypes::Enemy].into(),
        )
    }
}

impl From<Projectile> for MultiPlayerComponentOptions {
    fn from(value: Projectile) -> Self {
        Self {
            pred: true,
            interp: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ProjectileMovement {
    Linear(Vec2),
    Orbital {
        around: Entity,
        radius: f32,
        speed: f32,
        c_angle: f32,
    },
}

pub fn projectile_movement<QF: QueryFilter>(
    q_target: Query<&Position, Without<Projectile>>,
    mut q_projectile: Query<(&Projectile, &mut LinearVelocity), (With<Projectile>, QF)>,
) {
    for (proj, mut velo) in &mut q_projectile {
        match proj.movement {
            ProjectileMovement::Linear(v) => velo.0 = v,
            ProjectileMovement::Orbital {
                around,
                radius,
                speed,
                c_angle,
            } => {
                todo!()
            }
        }
    }
}

/// To be used whenever we're adding a projectile that needs the things that we don't network
pub fn add_projectile_components<QF: QueryFilter>(
    trigger: On<Add, Projectile>,
    mut commands: Commands,
    q_projectile: Query<(&Projectile), QF>,
) {
    if let Ok(p) = q_projectile.get(trigger.entity) {
        commands
            .entity(trigger.entity)
            .insert((CommonColliderBundle::from(*p)));
    }
}
