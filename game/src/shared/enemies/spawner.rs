use crate::{
    shared::{game_rules::MapKind, states::InGameTime},
    utils::{SpawnPattern, read_ron},
};

use super::*;
use bevy::{math::VectorSpace, prelude::*};

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct EnemySpawnManager {
    pub spawn_style: EnemySpawnStyle,
}

#[derive(Default, Reflect)]
pub enum EnemySpawnStyle {
    #[default]
    Automatic,
    Manual {
        instruction: EnemySpawnInstruction,
        should_fire: bool,
    },
    EditSpawnerWaves {
        level: MapKind,
        load: bool,
        save: bool,
        list: EnemySpawnerList,
    },
}

pub fn spawn_enemy_spawn_manager(mut commands: Commands) {
    commands.insert_resource(EnemySpawnManager {
        spawn_style: EnemySpawnStyle::Automatic,
    })
}

pub fn update_enemy_spawn_manager(mut commands: Commands, mut manager: ResMut<EnemySpawnManager>) {
    match manager.spawn_style {
        EnemySpawnStyle::Automatic => {}
        EnemySpawnStyle::Manual {
            instruction,
            ref mut should_fire,
        } => {
            if *should_fire {
                let positions = instruction.pattern.to_positions();
                for position in positions {
                    spawn_enemy(&mut commands, instruction.kind, position);
                }
                *should_fire = false;
            }
        }
        EnemySpawnStyle::EditSpawnerWaves {
            level,
            ref mut load,
            ref mut save,
            ref mut list,
        } => {
            if *load {
                let spawner_list = read_ron("assets/maps/grass/spawner.ron".into());
                *list = spawner_list;
                *load = false;
            }
            if *save {
                *save = false;
            }
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Default)]
pub struct EnemySpawnInstruction {
    pub kind: EnemyKind,
    pub pattern: SpawnPattern,
}

#[derive(Default, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Default)]
pub struct EnemySpawnerList(pub Vec<EnemySpawnerBuilder>);

#[derive(Debug, Serialize, Deserialize, Reflect)]
pub struct EnemySpawnerBuilder {
    activation: EnemySpawnerActivation,
    instruction: EnemySpawnInstruction,
}

impl From<EnemySpawnerBuilder> for EnemySpawner {
    fn from(value: EnemySpawnerBuilder) -> Self {
        let active_time = match value.activation {
            EnemySpawnerActivation::OneTime { at } => at,
            EnemySpawnerActivation::FixedWaves {
                start_time,
                tick_rate,
                max_ticks,
                c_ticks,
            } => start_time,
            EnemySpawnerActivation::TimeLimit {
                start_time,
                end_time,
                tick_rate,
            } => start_time,
        };
        Self {
            activation: value.activation,
            instruction: value.instruction,
            countdown_timer: Timer::from_seconds(active_time, TimerMode::Once),
            spawn_timer: None,
        }
    }
}

#[derive(Component, Debug, Serialize, Deserialize)]
pub struct EnemySpawner {
    activation: EnemySpawnerActivation,
    instruction: EnemySpawnInstruction,
    /// Fires after the countdown timer is finished.
    /// At that point, this will either be Some(Timer),
    /// or will be despawned
    spawn_timer: Option<Timer>,
    /// Spawned on init
    countdown_timer: Timer,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Default)]
pub enum EnemySpawnerActivation {
    OneTime {
        at: f32,
    },
    FixedWaves {
        start_time: f32,
        tick_rate: f32,
        max_ticks: u32,
        c_ticks: u32,
    },
    TimeLimit {
        start_time: f32,
        end_time: f32,
        tick_rate: f32,
    },
}
impl Default for EnemySpawnerActivation {
    fn default() -> Self {
        EnemySpawnerActivation::OneTime { at: 0.0 }
    }
}

pub fn add_enemy_spawner(mut commands: Commands) {
    commands.spawn(EnemySpawner {
        activation: EnemySpawnerActivation::TimeLimit {
            start_time: 5.0,
            end_time: 10.0,
            tick_rate: 1.0,
        },
        instruction: EnemySpawnInstruction {
            kind: EnemyKind::FacelessMan,
            pattern: SpawnPattern::Circle {
                amount: 10,
                center: Vec2::ZERO,
                radius: 100.0,
                radius_only: false,
            },
        },
        spawn_timer: None,
        countdown_timer: Timer::from_seconds(5.0, TimerMode::Once),
    });
}

pub fn update_enemy_spawner(
    mut commands: Commands,
    mut q_spawner: Query<(Entity, &mut EnemySpawner)>,
    game_timer: Res<Time<Virtual>>,
    in_game_time: Res<InGameTime>,
) {
    for (spawner_ent, mut spawner) in &mut q_spawner {
        let mut should_spawn_enemy = false;
        let mut should_despawn_self = false;
        // Handle the countdown initialization first
        spawner.countdown_timer.tick(game_timer.delta());
        if spawner.countdown_timer.just_finished() {
            let spawn_timer = match spawner.activation {
                EnemySpawnerActivation::FixedWaves {
                    start_time,
                    tick_rate,
                    max_ticks,
                    ref mut c_ticks,
                } => {
                    *c_ticks = max_ticks;
                    Some(Timer::from_seconds(tick_rate, TimerMode::Repeating))
                }
                EnemySpawnerActivation::TimeLimit {
                    start_time,
                    end_time,
                    tick_rate,
                } => Some(Timer::from_seconds(tick_rate, TimerMode::Repeating)),
                _ => None,
            };
            should_spawn_enemy = true;
            spawner.spawn_timer = spawn_timer
        } else {
            // Tick ongoing enemy timer
            if let Some(ref mut t) = spawner.spawn_timer {
                t.tick(game_timer.delta());
                let mut finished = false;
                if t.just_finished() {
                    finished = true;
                    should_spawn_enemy = true;
                }

                match spawner.activation {
                    EnemySpawnerActivation::FixedWaves {
                        start_time,
                        tick_rate,
                        max_ticks,
                        ref mut c_ticks,
                    } => {
                        if finished {
                            *c_ticks -= 1;
                            if *c_ticks == 0 {
                                should_despawn_self = true
                            }
                        }
                    }
                    EnemySpawnerActivation::TimeLimit {
                        start_time,
                        end_time,
                        tick_rate,
                    } => {
                        if end_time < in_game_time.0.elapsed_secs() {
                            should_despawn_self = true;
                        }
                    }
                    _ => {}
                }
            }
        }
        if should_spawn_enemy {
            let positions = spawner.instruction.pattern.to_positions();
            for pos in positions {
                spawn_enemy(&mut commands, spawner.instruction.kind, pos)
            }
        }
        if should_despawn_self {
            commands.entity(spawner_ent).despawn();
        }
    }
}
