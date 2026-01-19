use avian2d::prelude::*;
use bevy::{ecs::query::QueryFilter, prelude::*};
use lightyear::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    shared::{
        colliders::{ColliderTypes, CommonColliderBundle},
        game_kinds::*,
        game_object_spawning::*,
        players::Player,
    },
    utils::AssetFolder,
};

pub mod spawner;

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
            [
                ColliderTypes::Enemy,
                ColliderTypes::Player,
                ColliderTypes::PlayerProjectile,
            ]
            .into(),
        )
    }
}

impl From<Enemy> for MultiPlayerComponentOptions {
    fn from(value: Enemy) -> Self {
        Self {
            pred: true,
            interp: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Reflect, Default)]
#[reflect(Default)]
pub enum EnemyKind {
    #[default]
    FacelessMan,
}

impl From<EnemyKind> for AssetFolder {
    fn from(value: EnemyKind) -> Self {
        let string = match value {
            EnemyKind::FacelessMan => "enemies/faceless".into(),
        };
        Self(string)
    }
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

pub fn spawn_enemy(commands: &mut Commands, e_kind: EnemyKind, game_kind: GameKinds) {
    let enemy = Enemy {
        kind: e_kind,
        state: EnemyState::Spawning,
    };
    let mut rng = rand::rng();
    let pos = (rng.random_range(-50.0..50.0), rng.random_range(-50.0..50.0));
    let _ = spawn_game_object(
        commands,
        game_kind,
        MultiPlayerComponentOptions::from(enemy),
        (
            enemy,
            Position(Vec2::new(pos.0, pos.1)),
            EnemySpawnTimer::default(),
        ),
    );
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
                let timer = if m_timer.is_none() {
                    continue;
                } else {
                    m_timer.as_mut().unwrap()
                };
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

/// These add the components for enemies shortly after spawning.
/// This is run on both the server and the client to cover the things
/// that can't be replicated with one system
pub fn add_non_replicated_enemy_components<QF: QueryFilter>(
    trigger: On<Add, Enemy>,
    mut commands: Commands,
    q_to_attach: Query<&Enemy, (QF)>,
) {
    if let Ok(en) = q_to_attach.get(trigger.entity) {
        commands
            .entity(trigger.entity)
            .insert((EnemySpawnTimer::default(), CommonColliderBundle::from(*en)));
    }
}
