use avian2d::prelude::Position;
use bevy::prelude::*;

use crate::shared::{
    game_kinds::*,
    game_rules::GameRules,
    loading::LevelLoadingState,
    maps::*,
    pickups::{HealthPickup, HealthPickupSpawner, tmp_spawn_health_spawner},
    states::{AppState, InGameState, set_app_state_in_game},
    stats::xp::add_xp_manager,
};

#[cfg(feature = "dev")]
use crate::utils::zoo::*;

pub struct ClientGameLoadingPlugin;

impl Plugin for ClientGameLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LoadingLevel),
            add_map_loading_systems.run_if(is_single_player),
        )
        .add_systems(
            OnEnter(LevelLoadingState::LevelLoading),
            run_map_loading_systems.run_if(is_single_player),
        )
        .add_systems(
            OnEnter(LevelLoadingState::LevelReady),
            (set_app_state_in_game).run_if(in_state(AppState::LoadingLevel)),
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
