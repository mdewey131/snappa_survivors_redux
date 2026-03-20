use bevy::prelude::*;

use crate::shared::{
    game_kinds::*,
    game_rules::GameRules,
    loading::spawn_player_character,
    states::{AppState, InGameState},
    stats::xp::add_level_manager,
};

pub struct ClientGameLoadingPlugin;

impl Plugin for ClientGameLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LoadingLevel),
            (
                (spawn_player_character, add_level_manager).run_if(is_single_player),
                tmp_move_to_game,
            ),
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

/// For now, loading does nothing because I don't want to figure it out. Let's just get to the game stuff
fn tmp_move_to_game(
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<InGameState>>,
) {
    app_state.set(AppState::InGame);
    game_state.set(InGameState::InGame);
}
