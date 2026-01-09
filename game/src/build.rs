use std::time::Duration;

use bevy::{prelude::*, state::app::StatesPlugin};
use clap::{Parser, Subcommand};
use lightyear::prelude::{client::ClientPlugins, server::ServerPlugins};
use serde::Deserialize;

use crate::{
    client::{ClientRenderPlugin, GameClientPlugin},
    render::GameSharedRenderPlugin,
    server::{DedicatedServerPlugin, GameServerPlugin},
    shared::GameSharedPlugin,
};

const TICKRATE: f64 = 1.0 / 64.0;
/// Responsible for constructing the app when we launch the game via command line arguments
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    kind: Option<AppKind>,
}

#[derive(Subcommand, Debug, Clone, Copy)]
pub enum AppKind {
    Client {
        #[arg(short, long, default_value = None)]
        c_id: Option<u64>,
    },
    Server {
        #[arg(short, long)]
        render: bool,
    },
}

impl Cli {
    pub fn build_game_app(&self, app: &mut App) {
        let app_kind = self.kind.unwrap_or_else(|| panic!("App kind not found!"));
        match app_kind {
            AppKind::Client { c_id } => build_game_client_app(app, c_id, true),
            AppKind::Server { render } => build_game_server_app(app, render),
        }
    }
}

pub fn build_game_client_app(app: &mut App, c_id: Option<u64>, render: bool) {
    if render {
        add_bevy_default_app_plugins(app, "Client".into());
        add_shared_game_renderer(app);
        add_game_client_renderer(app);
    } else {
        add_headless_app_plugins(app);
    }
    // This first
    add_lightyear_client_plugin(app, TICKRATE);
    // per lightyear docs: add in protocol before spawning in client
    add_shared_game_plugin(app);

    add_game_client_plugin(app);
    // We want the ability to add a server to the client, because that's how single player will work
    add_game_server_plugin(app);
}

pub fn build_game_server_app(app: &mut App, render: bool) {
    if render {
        add_bevy_default_app_plugins(app, "Server".into());
        add_shared_game_renderer(app);
    } else {
        add_headless_app_plugins(app);
    }
    // This first
    add_lightyear_server_plugins(app, TICKRATE);
    // per lightyear docs: add in protocol before spawning in client
    add_shared_game_plugin(app);
    add_game_server_plugin(app);
    app.add_plugins(DedicatedServerPlugin);
}

pub fn add_lightyear_client_plugin(app: &mut App, tickrate: f64) {
    app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(tickrate),
    });
}

pub fn add_lightyear_server_plugins(app: &mut App, tickrate: f64) {
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(tickrate),
    });
}

pub fn add_game_client_plugin(app: &mut App) {
    app.add_plugins(GameClientPlugin);
}

pub fn add_game_client_renderer(app: &mut App) {
    app.add_plugins(ClientRenderPlugin);
}

pub fn add_game_server_plugin(app: &mut App) {
    app.add_plugins(GameServerPlugin);
}

pub fn add_shared_game_renderer(app: &mut App) {
    app.add_plugins(GameSharedRenderPlugin);
}

pub fn add_shared_game_plugin(app: &mut App) {
    app.add_plugins(GameSharedPlugin);
}

/// Used for servers and clients in non-render scenarios
pub fn add_headless_app_plugins(app: &mut App) {
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin));
}

pub fn add_bevy_default_app_plugins(app: &mut App, window_name: String) {
    // TODO: Use settings to read the resolution and other details that we need
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: window_name,
            ..default()
        }),
        ..default()
    }));
}
