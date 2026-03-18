use bevy::{ecs::system::SystemId, platform::collections::HashMap, prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    client::load_game::client_transition_to_loading_state,
    shared::{GameMainChannel, game_rules::GameRules, players::CharacterKind, states::AppState},
};

pub struct LobbyProtocolPlugin;

impl Plugin for LobbyProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<PlayerInLobby>().add_prediction();
        app.register_message::<ClientStartGameMessage>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<ServerStartLoadingGameMessage>()
            .add_direction(NetworkDirection::ServerToClient);
        app.add_message::<ClientStartGameMessage>();
        app.register_message::<ClientChangeCharacterMessage>()
            .add_direction(NetworkDirection::ClientToServer);
    }
}

/// A component that is only held on the server to manage the different peer ids.
/// The clients will primarily work with the `PlayerInLobby` idea, so that we don't
/// need to do any peer id stuff and can just get to the point in single player,
/// and so that we don't have to replicate this over the network in multiplayer
#[derive(Component, Debug, Clone)]
pub struct Lobby {
    pub players: [Option<Entity>; 8],
}

/// The game needs a way of representing each player in the lobby.
/// We can't just put the `Player` component on the entity, because
/// a variety of in game triggers rely on `Player` being inserted
/// with its stats. So, we have this representation that gets turned
/// into a player instead once we go live.
///
/// The other convenience is that this component can be put on the entity representing the client, and we'll just remove it
/// once its time to build a player from this
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[require(Replicate = Replicate::to_clients(NetworkTarget::All))]
pub struct PlayerInLobby {
    pub peer_id: PeerId,
    pub selected_character: Option<CharacterKind>,
    pub color: Color,
    pub name: String,
}

/// A marker component for the entity that has the power to make changes.
/// In multiplayer scenarios, this is used to selectively show ui elements,
/// and for the server to know who has the authority to do what (including transfer ownership of LobbyCaptain)
#[derive(Component, Debug, Clone)]
pub struct LobbyCaptain;

#[derive(Message, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClientChangeCharacterMessage {
    pub char: CharacterKind,
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
