use bevy::{ecs::query::QueryFilter, prelude::*};
use serde::{Deserialize, Serialize};

use crate::shared::{
    combat::CombatSystemSet,
    game_kinds::{CurrentGameKind, MultiPlayerComponentOptions},
    game_object_spawning::spawn_game_object,
    states::InGameState,
};

/// Lets us know that there has been a level up for players
#[derive(Message)]
pub struct LevelUpMessage;

pub struct SharedXPPlugin;
impl Plugin for SharedXPPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LevelUpMessage>().add_systems(
            FixedUpdate,
            (update_xp_manager)
                .in_set(CombatSystemSet::Combat)
                .run_if(in_state(InGameState::InGame)),
        );
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, PartialEq, Reflect)]
pub struct LevelManager {
    pub c_level: u8,
    pub c_xp: f32,
    /// For display purposes
    pub prev_max: f32,
    pub next_max: f32,
}

impl Default for LevelManager {
    fn default() -> Self {
        Self {
            c_level: 1,
            c_xp: 0.0,
            prev_max: 0.0,
            next_max: 5.0,
        }
    }
}

pub fn update_xp_manager(
    mut level_up_messages: MessageWriter<LevelUpMessage>,
    mut q_level: Single<&mut LevelManager>,
) {
    if q_level.c_xp >= q_level.next_max {
        level_up_messages.write(LevelUpMessage);
        q_level.c_level += 1;
        q_level.prev_max = q_level.next_max;
        q_level.next_max = (q_level.c_level as f32 * 5.0).powf(2.0);
    }
}

pub fn add_level_manager(mut commands: Commands, gk: Res<CurrentGameKind>) {
    let _lm = spawn_game_object(
        &mut commands,
        gk.0.unwrap(),
        None::<()>,
        MultiPlayerComponentOptions {
            pred: true,
            interp: false,
        },
        (LevelManager::default(), Name::from("Level Manager")),
    );
}
