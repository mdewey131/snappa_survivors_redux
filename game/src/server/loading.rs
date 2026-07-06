use crate::shared::{loading::LevelLoadingState, maps::*, states::*, stats::xp::add_xp_manager};
use bevy::prelude::*;

pub struct DedicatedServerLoadingPlugin;

impl Plugin for DedicatedServerLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LoadingLevel),
            (initialiize_map_builder, add_map_loading_systems),
        )
        .add_systems(
            OnEnter(LevelLoadingState::LevelLoading),
            run_map_loading_systems,
        )
        .add_systems(
            OnEnter(LevelLoadingState::LevelReady),
            (add_xp_manager, set_app_state_in_game)
                .chain()
                .run_if(in_state(AppState::LoadingLevel)),
        );
    }
}
