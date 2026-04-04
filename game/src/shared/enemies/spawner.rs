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
        EnemySpawnStyle::Automatic => {}
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



/// Describes the list of enemies that are available at a given moment in time (depends on the map) and their spawn weights
/// 
pub struct EnemySpawnTable {
    
}