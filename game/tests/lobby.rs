use bevy::state::state::State;
use lightyear::prelude::MessageReceiver;
use snappa_survivors::shared::{game_rules::*, states::AppState};
mod common;
use common::setup_test_client;

use crate::common::{move_to_single_player, setup_multiplayer_connected_apps, tick_app};

#[test]
fn change_rules_1p() -> Result<(), String> {
    let mut app = setup_test_client();
    // Burn in one update so that the startup happens.
    // If you don't do this, you'll move to EstablishServerConnection -> GameSplash
    app.update();
    let sys = app.register_system(send_game_change_message_callback::<Difficulty>);
    move_to_single_player(&mut app);
    for _updates in (0..30) {
        // Move to lobby
        tick_app(&mut app, 1.0 / 64.0);
    }
    let state = app.world().resource::<State<AppState>>().get().clone();
    // We should be in the lobby
    if matches!(state, AppState::Lobby) {
    } else {
        return Err(format!("Failed to reach lobby. Current state: {:?}", state));
    }

    app.world_mut()
        .run_system_with(sys, Difficulty::Hard)
        .expect("This should run");

    let rules = app.world().resource::<GameRules>();
    assert!(matches!(rules.difficulty, Difficulty::Hard));
    Ok(())
}

#[test]
fn change_rules_multiplayer() -> Result<(), String> {
    let (mut server, mut client) = setup_multiplayer_connected_apps();
    let sys = client.register_system(send_game_change_message_callback::<Difficulty>);
    for _updates in (0..30) {
        // Move to lobby
        tick_app(&mut server, 1.0 / 64.0);
        tick_app(&mut client, 1.0 / 64.0);
    }
    let c_state = client.world().resource::<State<AppState>>().get().clone();
    // We should be in the lobby
    if matches!(c_state, AppState::Lobby) {
    } else {
        return Err(format!(
            "Failed to reach lobby. Current state: {:?}",
            c_state
        ));
    }

    let s_state = server.world().resource::<State<AppState>>().get().clone();
    // We should be in the lobby
    if matches!(s_state, AppState::Lobby) {
    } else {
        return Err(format!(
            "Failed to reach lobby. Current state: {:?}",
            s_state
        ));
    }

    client
        .world_mut()
        .run_system_with(sys, Difficulty::Hard)
        .expect("This should run");

    for _update in (0..60) {
        tick_app(&mut client, 1.0 / 64.0);
        tick_app(&mut server, 1.0 / 64.0);
    }

    let server_rules = server.world().resource::<GameRules>();
    let client_rules = client.world().resource::<GameRules>();
    if matches!(client_rules.difficulty, Difficulty::Hard) {
    } else {
        return Err(format!(
            "Failed to match client difficulty. value found was {:?}",
            client_rules.difficulty
        ));
    }

    if matches!(server_rules.difficulty, Difficulty::Hard) {
        return Ok(());
    } else {
        return Err(format!(
            "Failed to match server difficulty. value found was {:?}",
            server_rules.difficulty
        ));
    }
}
