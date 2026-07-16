use bevy::prelude::*;
use lightyear::{connection::client::ClientState, prelude::Client};

use super::GameClient;
use crate::shared::{
    game_kinds::is_single_player,
    states::{AppState, InGameState, add_game_over_timer, check_game_over},
};
pub struct ClientStatesPlugin;
impl Plugin for ClientStatesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::EstablishServerConnection),
            ((GameClient::attempt_connection_to_server),).chain(),
        );
        app.add_systems(
            Update,
            transition_to_lobby
                .run_if(in_state(AppState::EstablishServerConnection).and_then(client_connected)),
        );
        app.add_systems(
            OnEnter(InGameState::InGame),
            add_game_over_timer.run_if(is_single_player),
        );
        app.add_systems(
            Update,
            check_game_over
                .run_if(is_single_player)
                .run_if(not(in_state(InGameState::OutOfGame))),
        );
    }
}

fn transition_to_lobby(mut state: ResMut<NextState<AppState>>) {
    state.set(AppState::Lobby)
}

fn client_connected(q_client: Single<&Client>) -> bool {
    match q_client.into_inner().state {
        ClientState::Connected => true,
        _ => false,
    }
}
