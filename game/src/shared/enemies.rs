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
            &Position,
            &mut LinearVelocity,
            Option<&mut EnemySpawnTimer>,
        ),
        (EnemyQF),
    >,
    q_targets: Query<(Entity, &Position), (With<Player>, Without<Enemy>, PlayerQF)>,
) {
    for (ent, mut enemy, e_pos, mut e_lv, mut m_timer) in &mut q_enemy {
        match enemy.state {
            EnemyState::Spawning => {
                let timer = m_timer.as_mut().unwrap();
                timer.0.tick(time.delta());
                if timer.0.just_finished() {
                    commands.entity(ent).remove::<EnemySpawnTimer>();
                    enemy.state = EnemyState::LookForTargets
                }
            }
            EnemyState::LookForTargets => {
                let closest = q_targets
                    .iter()
                    .min_by(|p1, p2| {
                        let d1 = p1.1.distance(e_pos.0);
                        let d2 = p2.1.distance(e_pos.0);
                        d1.total_cmp(&d2)
                    })
                    .map(|player| player.0);
                if let Some(p_ent) = closest {
                    enemy.state = EnemyState::MovingTo(p_ent)
                }
            }
            EnemyState::MovingTo(player) => {
                let e_ms = 30.0;
                if let Ok((_, p_pos)) = q_targets.get(player) {
                    let dir = (p_pos.0 - e_pos.0).normalize_or_zero();
                    e_lv.0 = dir * e_ms;
                } else {
                    enemy.state = EnemyState::LookForTargets
                }
            }
            EnemyState::Dying => {}
        }
    }
}
