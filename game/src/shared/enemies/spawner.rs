use crate::utils::SpawnPattern;

use super::*;
use bevy::prelude::*;

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
        kind: EnemyKind,
        should_fire: bool,
        pattern: SpawnPattern,
    },
}

pub fn spawn_enemy_spawn_manager(mut commands: Commands) {
    commands.insert_resource(EnemySpawnManager {
        spawn_style: EnemySpawnStyle::Automatic,
    })
}

pub fn update_enemy_spawn_manager(mut commands: Commands, mut manager: ResMut<EnemySpawnManager>) {
    match manager.spawn_style {
        EnemySpawnStyle::Automatic => {
            /*
            // Derive a difficulty score
            let n_players = q_players.iter().len() as f32;
            let time_mins = time.0.elapsed_secs();
            let n_enemies = q_enemies.iter().len() as f32;
            let diff_factor = match rules.difficulty {
                Difficulty::Easy => 5.0,
                Difficulty::Medium => 10.0,
                Difficulty::Hard => 15.0,
            };

            let target_diff = n_players * time_mins * diff_factor;

            let curr_diff = n_enemies * n_players * diff_factor;

            let to_spawn = (target_diff - curr_diff).floor() as i32;

            for _i in 0..to_spawn {
                let x: f32 = (2.0 * (rand::random::<f32>() - 0.5)) * 100.0;
                let y: f32 = (2.0 * (rand::random::<f32>() - 0.5)) * 100.0;
                let pos = Vec2::new(x, y);
                server_spawn_enemy(&mut commands, EnemyKind::default(), pos);
            }
            */
        }
        EnemySpawnStyle::Manual {
            kind,
            ref mut should_fire,
            pattern,
        } => {
            if *should_fire {
                let positions = pattern.to_positions();
                for position in positions {
                    spawn_enemy(&mut commands, kind, position);
                }
                *should_fire = false;
            }
        }
    }
}
