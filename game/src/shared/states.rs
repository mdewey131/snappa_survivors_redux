use bevy::prelude::*;

/// Handles all of the logic that is relevant to the game loop.
#[derive(States, Component, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum InGameState {
    #[default]
    OutOfGame,
    InGame,
    SelectingUpgrades,
    /// The game is not running, but it's not because we're selecting upgrades.
    Paused,
}

/// The different states of the app on the server and the client.
#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub enum AppState {
    #[default]
    AppInit,
    GameSplash,
    MainMenu,
    MultiplayerServerSelection,
    EstablishServerConnection,
    Lobby,
    LoadingLevel,
    InGame,
    PostGame,
}

/// Provides a state to open the pause menu. This is used so as to be
/// orthogonal to the in game state's idea of pausing, because
/// "it's an online game, you can't pause it" needs to be true on the client
#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum PauseState {
    #[default]
    Unpaused,
    Paused,
}

pub struct SharedStatesPlugin;
impl Plugin for SharedStatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>().init_state::<InGameState>();
    }
}
