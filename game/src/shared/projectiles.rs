use avian2d::prelude::*;
use bevy::{ecs::query::QueryFilter, prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

pub struct ProjectileProtocolPlugin;
impl Plugin for ProjectileProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Projectile>().add_prediction();
    }
}
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Projectile {
    movement: ProjectileMovement,
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
