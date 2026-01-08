use std::net::{Ipv4Addr, SocketAddr};

use crate::shared::{SHARED_SETTINGS, SINGLE_PLAYER_SERVER_PORT, SharedNetworkingSettings};
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use lightyear::{
    link::RecvLinkConditioner,
    netcode::NetcodeServer,
    prelude::{
        LocalAddr,
        server::{NetcodeConfig, ServerUdpIo},
    },
};
use serde::{Deserialize, Serialize};

pub struct GameServerPlugin;
impl Plugin for GameServerPlugin {
    fn build(&self, app: &mut App) {
        app;
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
