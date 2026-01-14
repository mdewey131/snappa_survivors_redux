use bevy::{ecs::system::SystemId, prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    client::load_game::client_transition_to_loading_state,
    shared::{GameMainChannel, game_rules::GameRules, states::AppState},
};

pub struct LobbyProtocolPlugin;

impl Plugin for LobbyProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_message::<ClientStartGameMessage>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<ServerStartLoadingGameMessage>()
            .add_direction(NetworkDirection::ServerToClient);
        app.add_message::<ClientStartGameMessage>();
    }
}

/// Sent from the client to the server to indicate that is time to start the game
#[derive(Message, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClientStartGameMessage;

/// Sent from the server to all clients to confirm that we're doing the thing, and its time to load a game
///
/// This is sent with information about the game rules for the reason that we don't want to trust that
/// clients will necessarily have the most up-to-date version of the game rules. Since gamerules is copy,
/// I'm thinking it's not too bad to write this off to each of the clients
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ServerStartLoadingGameMessage {
    pub rules: GameRules,
}
