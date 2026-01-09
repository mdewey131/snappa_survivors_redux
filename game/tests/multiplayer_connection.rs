mod common;
use lightyear::{
    connection::client::ClientState,
    prelude::{Client, Connect, server::Start},
};
use snappa_survivors::{client::game_client::GameClient, server::GameServer};
#[test]
fn multi_player_connection() -> Result<(), String> {
    // Spawn a server and the client on this app
    let mut client_app = common::setup_test_client();
    let mut server_app = common::setup_dedicated_server();

    server_app.update();
    // This works for now, but you probably will want a way to configure other settings in the future so that this can work more better
    let client = client_app.world_mut().spawn(GameClient::SINGLE_PLAYER).id();

    client_app
        .world_mut()
        .commands()
        .trigger(Connect { entity: client });

    client_app.update();

    let client_comp = client_app.world().get::<Client>(client).unwrap();
    match client_comp.state {
        ClientState::Disconnected | ClientState::Disconnecting => {
            Err(String::from("Not connected"))
        }
        ClientState::Connected | ClientState::Connecting => Ok(()),
    }
}
