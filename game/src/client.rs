use crate::{
    client::main_menu::MainMenuPlugin,
    shared::{game_kinds::CurrentGameKind, states::AppState},
};
use bevy::prelude::*;

pub mod game_client;
pub mod main_menu;

pub struct GameClientPlugin;
impl Plugin for GameClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, move_to_first_app_state);
    }
}

/// Handles some things that the client needs to render, but that a dedicated server never will
///
/// Examples of this would be the Splash Screen and the Main Menu, and there are probably
/// also some camera things that we'd want to do, but idk yet
pub struct ClientRenderPlugin;

impl Plugin for ClientRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MainMenuPlugin);
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

pub fn transition_to_single_player(
    mut commands: Commands,
    mut game_choice: ResMut<CurrentGameKind>,
) {
    
}
