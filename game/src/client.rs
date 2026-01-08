use crate::shared::{
    SERVER_ADDR, SERVER_PORT, SHARED_SETTINGS, SINGLE_PLAYER_SERVER_PORT, SharedNetworkingSettings,
};
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use lightyear::netcode::NetcodeClient;
use lightyear::{
    link::RecvLinkConditioner,
    prelude::{client::NetcodeConfig, *},
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

const SINGLE_PLAYER_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SINGLE_PLAYER_SERVER_PORT);
const SINGLE_PLAYER_CLIENT_PORT: u16 = 0;

pub struct GameClientPlugin;
impl Plugin for GameClientPlugin {
    fn build(&self, app: &mut App) {
        app;
    }
}

/// Denotes the entity that serves as the client for the game.
/// This component serves settings to Lightyear's client, but we
/// use this so that I can modify the settings and change elements of the
/// client with some easy commands, in the hope that doing it this way
/// is a sane solution to the single player problem in a way that doesn't
/// require conditional compilation or runtime arguments
#[derive(Component)]
#[component(immutable, on_insert = GameClient::on_insert)]
pub struct GameClient {
    // The ID of the client
    pub client_id: u64,
    // The client port to listen on for updates
    pub client_port: u16,
    // The server address that we want to find
    pub server_addr: SocketAddr,
    // An optional conditioner that helps simulate network conditions in
    // testing
    pub conditioner: Option<RecvLinkConditioner>,
    pub transport: GameClientTransports,
    pub shared: SharedNetworkingSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GameClientTransports {
    Udp,
    Steam,
}

impl GameClient {
    pub const SINGLE_PLAYER: Self = Self {
        client_id: 0,
        client_port: SINGLE_PLAYER_CLIENT_PORT,
        server_addr: SINGLE_PLAYER_ADDR,
        conditioner: None,
        transport: GameClientTransports::Udp,
        shared: SHARED_SETTINGS,
    };

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        world.commands().queue(move |world: &mut World| -> Result {
            let mut entity_mut = world.entity_mut(entity);
            let settings = entity_mut.take::<GameClient>().unwrap();
            let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), settings.client_port);
            entity_mut.insert((
                Client::default(),
                Link::new(settings.conditioner.clone()),
                LocalAddr(client_addr),
                PeerAddr(settings.server_addr),
                ReplicationReceiver::default(),
                PredictionManager::default(),
                Name::from("Client"),
            ));

            // Depending on the transport type, do a different thing here
            match settings.transport {
                GameClientTransports::Udp => {
                    let netcode = settings.add_netcode_client()?;
                    entity_mut.insert(netcode);
                }
                GameClientTransports::Steam => {
                    todo!()
                }
            }
            Ok(())
        });
    }

    fn add_netcode_client(&self) -> Result<NetcodeClient, BevyError> {
        let auth = Authentication::Manual {
            server_addr: self.server_addr,
            client_id: self.client_id,
            private_key: self.shared.private_key,
            protocol_id: self.shared.protocol_id,
        };
        let netcode_config = NetcodeConfig {
            // The server should time out clients when their connection is closed
            client_timeout_secs: 3,
            token_expire_secs: -1,
            ..default()
        };
        NetcodeClient::new(auth, netcode_config)
            .map_err(|_e| BevyError::from("Netcode Client not initialized!"))
    }
}
