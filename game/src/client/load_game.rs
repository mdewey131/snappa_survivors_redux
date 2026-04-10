use bevy::prelude::*;

use crate::shared::{
    game_kinds::*,
    game_rules::{GameRules, MapKind},
    loading::{LevelLoadingState, spawn_player_character},
    states::{AppState, InGameState, set_app_state_in_game},
    stats::xp::add_level_manager,
};

#[cfg(feature = "dev")]
use crate::utils::zoo::*;

pub struct ClientGameLoadingPlugin;

impl Plugin for ClientGameLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(LevelLoadingState::LevelReady),
            ((assemble_level), set_app_state_in_game).run_if(in_state(AppState::LoadingLevel)),
        );
    }
}

pub fn client_transition_to_loading_state(
    arg_in: In<GameRules>,
    mut rules: ResMut<GameRules>,
    mut state: ResMut<NextState<AppState>>,
) {
    *rules = *arg_in;
    state.set(AppState::LoadingLevel)
}

fn assemble_level(mut commands: Commands, game_kinds: Res<CurrentGameKind>, rules: Res<GameRules>) {
    match game_kinds.0.unwrap() {
        GameKinds::SinglePlayer => match rules.map_type {
            #[cfg(feature = "dev")]
            MapKind::DevZoo => {
                info!("Loading Level");
                let spawn_zoo_player_dummies = commands.register_system(spawn_zoo_characters);
                let spawn_zoo_weapons = commands.register_system(spawn_zoo_weapons);
                commands.run_system(spawn_zoo_player_dummies);
                commands.run_system(spawn_zoo_weapons);
            }
            _ => {
                let player_character_spawn_sys = commands.register_system(spawn_player_character);
                commands.run_system(player_character_spawn_sys);
                let level_manager_sys = commands.register_system(add_level_manager);
                commands.run_system(level_manager_sys);
            }
        },
        GameKinds::MultiPlayer => {}
    }
}
