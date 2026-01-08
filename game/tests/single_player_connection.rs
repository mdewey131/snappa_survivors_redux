mod common;
use lightyear::{
    connection::client::ClientState,
    prelude::{Client, Connect, server::Start},
};
use snappa_survivors::{client::GameClient, server::GameServer};
#[test]
fn single_player_connection() -> Result<(), String> {
    // Spawn a server and the client on this app
    let mut app = common::setup_test_client();

    let client = app.world_mut().spawn(GameClient::SINGLE_PLAYER).id();
    let server = app.world_mut().spawn(GameServer::SINGLE_PLAYER).id();

    app.world_mut().commands().trigger(Start { entity: server });

    app.update();
    app.world_mut()
        .commands()
        .trigger(Connect { entity: client });

    app.update();
    let client_comp = app.world().get::<Client>(client).unwrap();
    match client_comp.state {
        ClientState::Disconnected | ClientState::Disconnecting => {
            Err(String::from("Not connected"))
        }
        ClientState::Connected | ClientState::Connecting => Ok(()),
    }
}
