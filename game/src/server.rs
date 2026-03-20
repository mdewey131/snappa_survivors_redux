use std::net::{Ipv4Addr, SocketAddr};

use crate::{
    server::{
        enemies::{DedicatedServerEnemyPlugin, ServerEnemyRenderPlugin},
        game_rules::DedicatedServerGameRulesPlugin,
        lobby::DedicatedServerLobbyPlugin,
        players::ServerPlayerRenderPlugin,
        weapons::*,
    },
    shared::{
        SEND_INTERVAL, SERVER_PORT, SHARED_SETTINGS, SINGLE_PLAYER_SERVER_PORT,
        SharedNetworkingSettings,
        game_kinds::CurrentGameKind,
        lobby::{Lobby, LobbyCaptain, PlayerInLobby},
        states::AppState,
        upgrades::DedicatedServerUpgradePlugin,
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
        *,
    },
};
use serde::{Deserialize, Serialize};
mod enemies;
mod game_rules;
mod loading;
mod lobby;
mod players;
mod projectiles;
mod weapons;

use loading::DedicatedServerLoadingPlugin;
use players::ServerPlayerPlugin;
use projectiles::DedicatedServerProjectilePlugin;

pub struct GameServerPlugin;
impl Plugin for GameServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ServerPlayerPlugin))
            .add_observer(handle_new_client)
            .add_observer(add_player_to_lobby);
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
        app.add_plugins((
            DedicatedServerEnemyPlugin,
            DedicatedServerGameRulesPlugin,
            DedicatedServerLobbyPlugin,
            DedicatedServerLoadingPlugin,
            DedicatedServerProjectilePlugin,
            DedicatedServerWeaponsPlugin,
            DedicatedServerUpgradePlugin,
        ))
        .add_systems(Startup, server_startup)
        .add_systems(OnEnter(AppState::Lobby), update_game_kind_resource);
    }
}

/// In cases where we have a dedicated server and for visual inspection, we're going to want to have some bare amount of rendering that
/// is special for just this server
pub struct DedicatedServerRendererPlugin;
impl Plugin for DedicatedServerRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ServerPlayerRenderPlugin,
            ServerEnemyRenderPlugin,
            DedicatedServerWeaponsRenderPlugin,
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

pub fn handle_new_client(
    trigger: On<Add, LinkOf>,
    mut commands: Commands,
    q_peer: Query<&RemoteId>,
) {
    let client = commands.entity(trigger.entity).insert((
        ReplicationSender::new(
            SEND_INTERVAL,
            lightyear::prelude::SendUpdatesMode::SinceLastAck,
            false,
        ),
        ReplicationReceiver::default(),
    ));
}

fn add_player_to_lobby(
    trigger: On<Add, RemoteId>,
    mut commands: Commands,
    q_peer: Query<&RemoteId>,
    mut q_lobby: Single<&mut Lobby>,
) {
    if let Ok(p_id) = q_peer.get(trigger.entity) {
        let player_pos = q_lobby.add_player(trigger.entity);
        commands.entity(trigger.entity).insert(PlayerInLobby {
            peer_id: p_id.0,
            selected_character: None,
            color: Color::srgb(1.0, 0.7, 0.7),
            name: format!("{:?}", p_id),
        });
        if let Some(0) = player_pos {
            commands.entity(trigger.entity).insert(LobbyCaptain);
        }
    }
}

fn update_game_kind_resource(mut r: ResMut<CurrentGameKind>) {
    r.0 = Some(crate::shared::game_kinds::GameKinds::MultiPlayer);
}
