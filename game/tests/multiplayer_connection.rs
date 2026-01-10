mod common;
use lightyear::{connection::client::ClientState, prelude::Client};

use crate::common::setup_multiplayer_connected_apps;
#[test]
fn multi_player_connection() -> Result<(), String> {
    let (mut _server, mut client_app) = setup_multiplayer_connected_apps();

    let mut q_client = client_app.world_mut().query::<&Client>();
    let client_comp = q_client
        .single(client_app.world_mut())
        .expect("This should exist");
    match client_comp.state {
        ClientState::Disconnected | ClientState::Disconnecting | ClientState::Connecting => {
            Err(String::from("Not connected"))
        }
        ClientState::Connected => Ok(()),
    }
}
