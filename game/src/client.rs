use crate::{
    client::{
        enemies::ClientEnemyRenderPlugin,
        game_client::{GameClient, GameClientConfig},
        load_game::ClientGameLoadingPlugin,
        lobby::ClientGameLobbyPlugin,
        main_menu::MainMenuPlugin,
        mp_selection_menu::MPSelectionMenuPlugin,
        players::ClientPlayerRenderPlugin,
    },
    shared::{
        SEND_INTERVAL,
        game_kinds::{CurrentGameKind, GameKinds, SinglePlayer},
        states::AppState,
        upgrades::ClientUpgradePlugin,
    },
};
use bevy::prelude::*;
use lightyear::prelude::{Client, Predicted, ReplicationSender, Server, Timeline};

pub mod camera;
pub mod client_states;
pub mod enemies;
pub mod game_client;
pub mod load_game;
pub mod lobby;
pub mod main_menu;
pub mod mp_selection_menu;
pub mod players;
pub mod projectiles;
mod weapons;
use camera::GameCameraClientPlugin;
use client_states::ClientStatesPlugin;
use enemies::ClientEnemyPlugin;
use players::ClientPlayerPlugin;
use projectiles::ClientProjectilePlugin;
use weapons::*;

pub struct GameClientPlugin;
impl Plugin for GameClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ClientEnemyPlugin,
            ClientStatesPlugin,
            ClientGameLobbyPlugin,
            ClientGameLoadingPlugin,
            ClientPlayerPlugin,
            ClientProjectilePlugin,
            ClientWeaponsPlugin,
            ClientUpgradePlugin,
        ))
        .add_systems(Startup, move_to_first_app_state)
        .add_observer(add_input_delay_on_client_add);
    }
}

/// Handles some things that the client needs to render, but that a dedicated server never will
///
/// Examples of this would be the Splash Screen and the Main Menu, and there are probably
/// also some camera things that we'd want to do, but idk yet
pub struct ClientRenderPlugin;

impl Plugin for ClientRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GameCameraClientPlugin,
            ClientEnemyRenderPlugin,
            MainMenuPlugin,
            MPSelectionMenuPlugin,
            ClientPlayerRenderPlugin,
            ClientDiceGuardRenderPlugin,
        ));
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
    q_client: Option<Single<Entity, With<Client>>>,
    q_server: Option<Single<Entity, With<Server>>>,
) {
    if let Some(c) = q_client {
        commands.entity(*c).despawn();
    }
    if let Some(s) = q_server {
        commands.entity(*s).despawn();
    }
    /*
    commands.spawn((GameClient::SINGLE_PLAYER));
    commands.spawn(GameServer::SINGLE_PLAYER);
    */
    game_choice.0 = Some(GameKinds::SinglePlayer);
    state.set(AppState::Lobby);
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

fn add_input_delay_on_client_add(trigger: On<Add, Client>, mut commands: Commands) {
    use lightyear::prelude::{
        Input,
        client::{InputDelayConfig, InputTimeline},
    };
    let input = Input::default().with_input_delay(InputDelayConfig::fixed_input_delay(10));

    commands.entity(trigger.entity).insert((
        ReplicationSender::new(
            SEND_INTERVAL,
            lightyear::prelude::SendUpdatesMode::SinceLastAck,
            false,
        ),
        InputTimeline(Timeline {
            context: input,
            ..default()
        }),
    ));
}
