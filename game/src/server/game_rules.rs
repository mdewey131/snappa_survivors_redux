use crate::shared::{game_rules::*, states::AppState};
use bevy::prelude::*;
pub struct DedicatedServerGameRulesPlugin;
impl Plugin for DedicatedServerGameRulesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                receive_game_change_message::<Difficulty>,
                receive_game_change_message::<MapKind>,
            )
                .run_if(in_state(AppState::Lobby)),
        );
    }
}
