use bevy::prelude::*;
use lightyear::{
    connection::client::ClientState,
    prelude::{Client, Connect, server::Start},
};

mod common;
use common::{move_to_single_player, tick_app};

#[test]
fn single_player_connection() -> Result<(), String> {
    // Spawn a server and the client on this app
    let mut app = common::setup_test_client();

    move_to_single_player(&mut app);
    // Run a few updates just to make sure this takes
    for updates in (0..30) {
        tick_app(&mut app, 1.0 / 64.0);
    }

    let mut q_client = app.world_mut().query::<&Client>();
    let client_comp = q_client
        .single(app.world_mut())
        .expect("There should be exactly one client");

    match client_comp.state {
        ClientState::Disconnected | ClientState::Disconnecting | ClientState::Connecting => {
            Err(format!(
                "Not connected. Actual client state: {:?}",
                client_comp.state,
            ))
        }
        ClientState::Connected => Ok(()),
    }
}
