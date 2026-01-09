use bevy::prelude::*;
use lightyear::{
    connection::client::ClientState,
    prelude::{Client, Server, server::Start},
};

use super::GameClient;
use crate::shared::{
    game_kinds::{CurrentGameKind, GameKinds},
    states::AppState,
};
pub struct ClientStatesPlugin;
impl Plugin for ClientStatesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::EstablishServerConnection),
            (
                start_1p_server.run_if(single_player),
                GameClient::attempt_connection_to_server,
            )
                .chain(),
        );
        app.add_systems(
            Update,
            transition_to_lobby
                .run_if(in_state(AppState::EstablishServerConnection).and(client_connected)),
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

fn start_1p_server(
    mut commands: Commands,
    res: Res<CurrentGameKind>,
    q_server: Single<Entity, With<Server>>,
) {
    info!("Current Game Kind: {:?}", res.0.unwrap());
    commands.trigger(Start { entity: *q_server })
}

fn single_player(res: Res<CurrentGameKind>) -> bool {
    match res.0.unwrap() {
        GameKinds::SinglePlayer => true,
        _ => false,
    }
}
