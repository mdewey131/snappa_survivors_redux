use avian2d::prelude::*;
use bevy::{ecs::query::QueryFilter, prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shared::{
    colliders::{ColliderTypes, CommonColliderBundle},
    players::Player,
};

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Reflect)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub state: EnemyState,
}

impl From<Enemy> for CommonColliderBundle {
    fn from(value: Enemy) -> Self {
        Self::new(
            RigidBody::Dynamic,
            Collider::capsule(20.0, 30.0),
            1.0,
            [ColliderTypes::Enemy].into(),
            [ColliderTypes::Enemy, ColliderTypes::Player].into(),
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Reflect)]
pub enum EnemyKind {
    FacelessMan,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Reflect)]
pub enum EnemyState {
    Spawning,
    LookForTargets,
    MovingTo(Entity),
    Dying,
}

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
pub struct EnemySpawnTimer(pub Timer);
impl Default for EnemySpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.5, TimerMode::Once))
    }
}

pub struct EnemyProtocolPlugin;

impl Plugin for EnemyProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Enemy>();
    }
}

pub fn enemy_state_machine<EnemyQF: QueryFilter, PlayerQF: QueryFilter>(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut q_enemy: Query<
        (
            Entity,
            &mut Enemy,
            &mut Position,
            Option<&mut EnemySpawnTimer>,
        ),
        (EnemyQF),
    >,
    q_targets: Query<(Entity, &Position), (With<Player>, Without<Enemy>, PlayerQF)>,
) {
    for (ent, mut enemy, mut e_pos, mut m_timer) in &mut q_enemy {
        match enemy.state {
            EnemyState::Spawning => {
                let timer = m_timer.as_mut().unwrap();
                timer.0.tick(time.delta());
                if timer.0.just_finished() {
                    commands.entity(ent).remove::<EnemySpawnTimer>();
                    enemy.state = EnemyState::LookForTargets
                }
            }
            EnemyState::LookForTargets => {}
            EnemyState::MovingTo(p_pos) => {}
            EnemyState::Dying => {}
        }
    }
}
