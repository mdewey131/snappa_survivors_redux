use bevy::{ecs::entity::MapEntities, prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shared::{game_rules::GameRules, players::CharacterKind};

pub struct LobbyProtocolPlugin;

impl Plugin for LobbyProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.component::<Lobby>().predict();

        app.component::<LobbyCaptain>();
        app.component::<PlayerInLobby>().predict();

        app.add_message::<ClientStartGameMessage>();
        app.register_message::<ClientStartGameMessage>()
            .add_direction(NetworkDirection::ClientToServer);

        app.register_message::<ServerStartLoadingGameMessage>()
            .add_direction(NetworkDirection::ServerToClient);

        app.add_message::<ClientChangeCharacterMessage>();
        app.register_message::<ClientChangeCharacterMessage>()
            .add_direction(NetworkDirection::ClientToServer);
    }
}

/// The component that is used to understand which players are in the lobby, and in what order.
/// It's generally expected that the first person in will be the lobby captain when initializing,
/// but we're not enforcing that with the lobby because this can change
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lobby {
    pub players: Vec<Entity>,
    pub max_players: usize,
}

impl Lobby {
    pub fn add_player(&mut self, ent: Entity) -> Option<usize> {
        let lobby_has_capacity = self.max_players > self.players.len();
        let player_not_in_lobby = !self.players.contains(&ent);
        if lobby_has_capacity && player_not_in_lobby {
            self.players.push(ent);
            let new_len = self.players.len();
            Some(new_len - 1)
        } else {
            None
        }
    }
    pub fn rm_player(&mut self, ent: Entity) {
        let player_pos = self.players.iter().position(|e| *e == ent);
        if let Some(p) = player_pos {
            self.players.remove(p);
        }
    }
}

impl MapEntities for Lobby {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        self.players = self
            .players
            .iter()
            .map(|e| entity_mapper.get_mapped(*e))
            .collect::<Vec<Entity>>()
    }
}

/// The game needs a way of representing each player in the lobby.
/// We can't just put the `Player` component on the entity, because
/// a variety of in game triggers rely on `Player` being inserted
/// with its stats. So, we have this representation that gets turned
/// into a player instead once we go live.
///
/// The other convenience is that this component can be put on the entity representing the client, and we'll just remove it
/// once its time to build a player from this
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
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
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
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
