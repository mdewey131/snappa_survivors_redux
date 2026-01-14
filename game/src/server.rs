use std::net::{Ipv4Addr, SocketAddr};

use crate::{
    render::{enemies::EnemyRenderPlugin, player::PlayerRenderPlugin},
    server::enemies::DedicatedServerEnemyPlugin,
    shared::{
        SEND_INTERVAL, SERVER_PORT, SHARED_SETTINGS, SINGLE_PLAYER_SERVER_PORT,
        SharedNetworkingSettings, game_rules::ServerGameRulesPlugin, states::AppState,
    },
};
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use lightyear::{
    link::RecvLinkConditioner,
    netcode::NetcodeServer,
    prelude::{
        LinkOf, LocalAddr, Replicate, ReplicationReceiver, ReplicationSender,
        server::{NetcodeConfig, ServerUdpIo, Start},
    },
};
use serde::{Deserialize, Serialize};
mod enemies;
mod loading;
mod lobby;
mod players;

use enemies::ServerEnemyPlugin;
use loading::ServerLoadingPlugin;
use lobby::ServerLobbyPlugin;
use players::ServerPlayerPlugin;

pub struct GameServerPlugin;
impl Plugin for GameServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ServerGameRulesPlugin,
            ServerEnemyPlugin,
            ServerLobbyPlugin,
            ServerLoadingPlugin,
            ServerPlayerPlugin,
        ))
        .add_observer(handle_new_client);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ServerTransports {
    Udp { local_port: u16 },
    Steam { local_port: u16 },
}

#[derive(Component, Debug)]
#[component(on_insert = GameServer::on_insert)]
pub struct GameServer {
    pub conditioner: Option<RecvLinkConditioner>,
    pub transport: ServerTransports,
    pub shared: SharedNetworkingSettings,
}

impl GameServer {
    pub const SINGLE_PLAYER: Self = Self {
        conditioner: None,
        transport: ServerTransports::Udp {
            local_port: SINGLE_PLAYER_SERVER_PORT,
        },
        shared: SHARED_SETTINGS,
    };
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let ent = context.entity;
        world.commands().queue(move |world: &mut World| -> Result {
            let mut entity_mut = world.entity_mut(ent);
            let settings = entity_mut.take::<GameServer>().unwrap();
            entity_mut.insert(Name::from("Server"));
            match settings.transport {
                ServerTransports::Udp { local_port } => {
                    let server = settings.add_netcode_server();
                    entity_mut.insert((
                        LocalAddr(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), local_port)),
                        ServerUdpIo::default(),
                        server,
                    ));
                }
                ServerTransports::Steam { local_port } => {
                    todo!()
                }
            }
            Ok(())
        })
    }

    fn add_netcode_server(&self) -> NetcodeServer {
        let pk = self.shared.private_key;
        NetcodeServer::new(NetcodeConfig {
            protocol_id: self.shared.protocol_id,
            private_key: pk,
            ..default()
        })
    }
}

/// Only to be used when we're launching a dedicated server. This moves along some of the game state so that the server is in a place where its ready to
/// accept connections
pub struct DedicatedServerPlugin;

impl Plugin for DedicatedServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DedicatedServerEnemyPlugin)
            .add_systems(Startup, server_startup);
    }
}

/// In cases where we have a dedicated server and for visual inspection, we're going to want to have some bare amount of rendering that
/// is special for just this server
pub struct DedicatedServerRendererPlugin;
impl Plugin for DedicatedServerRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PlayerRenderPlugin::<Replicate>::new(),
            EnemyRenderPlugin::<Replicate>::new(),
        ));
    }
}

/// A startup system that creates the game server in a dedicated scenario.
/// In the future, this should be something that can be created and called back to
fn server_startup(mut commands: Commands, mut state: ResMut<NextState<AppState>>) {
    let server = GameServer {
        conditioner: None,
        transport: ServerTransports::Udp {
            local_port: SERVER_PORT,
        },
        shared: SHARED_SETTINGS,
    };
    let server_ent = commands.spawn(server).id();
    commands.trigger(Start { entity: server_ent });
    state.set(AppState::Lobby);
}

pub fn handle_new_client(trigger: On<Add, LinkOf>, mut commands: Commands) {
    commands.entity(trigger.entity).insert((
        ReplicationSender::new(
            SEND_INTERVAL,
            lightyear::prelude::SendUpdatesMode::SinceLastAck,
            false,
        ),
        ReplicationReceiver::default(),
    ));
}
