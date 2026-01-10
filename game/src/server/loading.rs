use crate::shared::states::*;
use bevy::prelude::*;

pub struct ServerLoadingPlugin;

impl Plugin for ServerLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::LoadingLevel), tmp_move_to_game);
    }
}

fn tmp_move_to_game(
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<InGameState>>,
) {
    app_state.set(AppState::InGame);
    game_state.set(InGameState::InGame);
}
