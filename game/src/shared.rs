use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use avian2d::prelude::*;
use bevy::prelude::*;
use lightyear::{
    avian2d::plugin::{AvianReplicationMode, LightyearAvianPlugin},
    prelude::*,
};

pub mod colliders;
pub mod combat;
pub mod despawn_timer;
pub mod enemies;
pub mod game_kinds;
pub mod game_object_spawning;
pub mod game_rules;
pub mod inputs;
pub mod lobby;
pub mod players;
pub mod projectiles;
pub mod states;
pub mod weapons;

use colliders::*;
use combat::CombatPlugin;
use despawn_timer::DespawnTimerPlugin;
use enemies::EnemyProtocolPlugin;
use game_kinds::GameKindsPlugin;
use game_rules::SharedGameRulesPlugin;
use inputs::GameInputProtocolPlugin;
use lobby::LobbyProtocolPlugin;
use projectiles::ProjectileProtocolPlugin;
use states::SharedStatesPlugin;
use weapons::{SharedWeaponPlugin, WeaponProtocolPlugin};

use crate::{
    shared::players::{Player, PlayerProtocolPlugin},
    utils::CreatedBy,
};

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
pub const SEND_INTERVAL: Duration = Duration::from_millis(100);

/// Has all shared logic for the game, separate from rendering concerns
pub struct GameSharedPlugin;

impl Plugin for GameSharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CombatPlugin,
            DespawnTimerPlugin,
            GameProtocolPlugin,
            GameKindsPlugin,
            SharedStatesPlugin,
            SharedGameRulesPlugin,
            SharedWeaponPlugin,
        ));

        app.add_plugins((
            LightyearAvianPlugin {
                replication_mode: AvianReplicationMode::Position,
                ..default()
            },
            PhysicsPlugins::new(FixedPostUpdate)
                .with_length_unit(1.0)
                .build()
                .disable::<ColliderTransformPlugin>()
                // Lightyear handles this
                .disable::<PhysicsTransformPlugin>()
                .disable::<PhysicsInterpolationPlugin>()
                // Disable this per https://discord.com/channels/691052431525675048/1189344685546811564/1450484188867330129
                // basically, this doesn't play well with rollbacks at all, and causes issues down the line
                .disable::<IslandPlugin>()
                .disable::<IslandSleepingPlugin>(),
        ))
        .insert_resource(Gravity::ZERO);
    }
}

struct GameProtocolPlugin;
impl Plugin for GameProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EnemyProtocolPlugin,
            LobbyProtocolPlugin,
            PlayerProtocolPlugin,
            GameInputProtocolPlugin,
            ProjectileProtocolPlugin,
            WeaponProtocolPlugin,
        ))
        .add_channel::<GameMainChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);

        app.register_component::<Position>()
            .add_prediction()
            .add_should_rollback(position_should_rollback)
            .add_linear_interpolation()
            .add_linear_correction_fn();
        app.register_component::<CreatedBy>().add_map_entities();
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SharedNetworkingSettings {
    /// An id to identify the protocol version
    pub protocol_id: u64,

    /// a 32-byte array to authenticate via the Netcode.io protocol
    pub private_key: [u8; 32],
}

#[derive(Debug)]
pub struct GameMainChannel;

fn position_should_rollback(this: &Position, that: &Position) -> bool {
    (this.0 - that.0).length() >= 0.05
}
