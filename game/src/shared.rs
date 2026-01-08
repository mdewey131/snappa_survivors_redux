use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use avian2d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::*;

mod combat;
mod states;
use combat::CombatPlugin;
use states::SharedStatesPlugin;

pub const SHARED_SETTINGS: SharedNetworkingSettings = SharedNetworkingSettings {
    protocol_id: 0,
    private_key: [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ],
};
pub const SERVER_PORT: u16 = 5888;
pub const SINGLE_PLAYER_SERVER_PORT: u16 = 5888;
/// 0 means that the OS will assign any available port
pub const CLIENT_PORT: u16 = 0;
pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SERVER_PORT);

/// Has all shared logic for the game, separate from rendering concerns
pub struct GameSharedPlugin;

impl Plugin for GameSharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((GameProtocolPlugin, CombatPlugin, SharedStatesPlugin));
    }
}

struct GameProtocolPlugin;
impl Plugin for GameProtocolPlugin {
    fn build(&self, app: &mut App) {
        app;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SharedNetworkingSettings {
    /// An id to identify the protocol version
    pub protocol_id: u64,

    /// a 32-byte array to authenticate via the Netcode.io protocol
    pub private_key: [u8; 32],
}
