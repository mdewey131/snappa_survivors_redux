use crate::{
    client::{
        game_client::{GameClient, GameClientConfig},
        load_game::ClientGameLoadingPlugin,
        main_menu::MainMenuPlugin,
        mp_selection_menu::MPSelectionMenuPlugin,
    },
    server::GameServer,
    shared::{
        game_kinds::{CurrentGameKind, GameKinds},
        states::AppState,
    },
};
use bevy::prelude::*;

pub mod client_states;
pub mod game_client;
pub mod load_game;
pub mod main_menu;
pub mod mp_selection_menu;
use client_states::ClientStatesPlugin;

pub struct GameClientPlugin;
impl Plugin for GameClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ClientStatesPlugin, ClientGameLoadingPlugin))
            .add_systems(Startup, move_to_first_app_state);
    }
}

/// Handles some things that the client needs to render, but that a dedicated server never will
///
/// Examples of this would be the Splash Screen and the Main Menu, and there are probably
/// also some camera things that we'd want to do, but idk yet
pub struct ClientRenderPlugin;

impl Plugin for ClientRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MainMenuPlugin, MPSelectionMenuPlugin));
    }
}

/// We enter in AppState::AppInit. Next, we want to move on to a state that
/// makes sense. I'm going to eventually want to have a splash screen setup, but
/// we can skip over that with a FF for the time being
fn move_to_first_app_state(mut state: ResMut<NextState<AppState>>) {
    if cfg!(feature = "no_splash") {
        state.set(AppState::MainMenu);
    } else {
        state.set(AppState::GameSplash)
    }
}

/// When we make the move to single player, we need to spawn both a game client and a game server.
/// The state AppState::AwaitingServerConnection will be the thing that actually attempts to make
/// the connection to the server, and that state will be responsible for figuring out when its time
/// to move to the lobby
pub fn transition_to_single_player(
    mut commands: Commands,
    mut game_choice: ResMut<CurrentGameKind>,
    mut state: ResMut<NextState<AppState>>,
) {
    commands.spawn((GameClient::SINGLE_PLAYER));
    commands.spawn(GameServer::SINGLE_PLAYER);
    game_choice.0 = Some(GameKinds::SinglePlayer);
    state.set(AppState::EstablishServerConnection);
}

pub fn transition_to_multi_player(
    config: In<GameClientConfig>,
    mut commands: Commands,
    mut game_choice: ResMut<CurrentGameKind>,
    mut state: ResMut<NextState<AppState>>,
) {
    commands.spawn(GameClient { config: config.0 });
    game_choice.0 = Some(GameKinds::MultiPlayer);
    state.set(AppState::EstablishServerConnection);
}
