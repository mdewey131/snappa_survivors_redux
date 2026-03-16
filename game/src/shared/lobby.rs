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
        app.register_component::<PlayerInLobby>();
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
    pub players: HashMap<PeerId, Entity>,
    current_players: u8,
    max_players: u8,
}

impl Lobby {
    pub fn add_player(&mut self, p_id: &PeerId, commands: &mut Commands) -> Option<Entity> {
        if self.current_players < self.max_players {
            let new_player = commands
                .spawn((
                    PlayerInLobby {
                        peer_id: *p_id,
                        selected_character: None,
                        color: Color::srgb(1.0, 0.7, 0.7),
                        name: format!("{:?}", p_id),
                    },
                    Replicate::to_clients(NetworkTarget::All),
                ))
                .id();
            self.current_players += 1;
            Some(new_player)
        } else {
            None
        }
    }
}

/// The game needs a way of representing each player in the lobby.
/// We can't just put the `Player` component on the entity, because
/// a variety of in game triggers rely on `Player` being inserted
/// with its stats. So, we have this representation that gets turned
/// into a player instead
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerInLobby {
    peer_id: PeerId,
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
