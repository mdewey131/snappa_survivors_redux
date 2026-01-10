use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

pub struct LobbyProtocolPlugin;

impl Plugin for LobbyProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_message::<ClientStartGame>()
            .add_direction(NetworkDirection::ClientToServer);
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClientStartGame;

#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct StartGameCountdown(Timer);
