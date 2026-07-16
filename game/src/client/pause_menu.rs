use crate::{
    render::menus::in_game_pause_menu::*,
    shared::{game_kinds::is_single_player, states::*},
};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
pub struct ClientPauseMenuPlugin;

impl Plugin for ClientPauseMenuPlugin {
    fn build(&self, app: &mut App) {
        // Pause menu when in multi player, just need this
        app.add_systems(
            Update,
            (
                (spawn_pause_menu).run_if(
                    in_state(AppState::InGame)
                        .and_then(input_just_pressed(KeyCode::Escape))
                        .and_then(|q_screen: Option<Single<&PauseMenuScreen>>| q_screen.is_none())
                        .and_then(not(is_single_player)),
                ),
                (despawn_pause_menu).run_if(
                    in_state(AppState::InGame)
                        .and_then(|q_screen: Option<Single<&PauseMenuScreen>>| q_screen.is_some())
                        .and_then(input_just_pressed(KeyCode::Escape))
                        .and_then(not(is_single_player)),
                ),
            ),
        );

        // When in single player, you also want to pause the game state
        app.add_systems(
            Update,
            (
                (spawn_pause_menu, pause_in_game_state).run_if(
                    in_state(AppState::InGame)
                        .and_then(input_just_pressed(KeyCode::Escape))
                        .and_then(|q_screen: Option<Single<&PauseMenuScreen>>| q_screen.is_none())
                        .and_then(is_single_player),
                ),
                (
                    despawn_pause_menu,
                    unpause_in_game_state.run_if(resource_exists::<InGamePauseManager>),
                )
                    .run_if(
                        in_state(AppState::InGame)
                            .and_then(|q_screen: Option<Single<&PauseMenuScreen>>| {
                                q_screen.is_some()
                            })
                            .and_then(input_just_pressed(KeyCode::Escape))
                            .and_then(is_single_player),
                    ),
            ),
        );
    }
}
