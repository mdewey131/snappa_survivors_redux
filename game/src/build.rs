use bevy::prelude::*;
use clap::{Parser, Subcommand};

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

pub fn build_game_client_app(app: &mut App, c_id: Option<u64>, render: bool) {}

pub fn build_game_server_app(app: &mut App, render: bool) {}
