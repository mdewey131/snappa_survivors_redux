use bevy::prelude::*;
use snappa_survivors::build::{build_game_client_app, build_game_server_app};

pub fn setup_test_client() -> App {
    let mut app = App::new();
    build_game_client_app(&mut app, None, false);
    app
}
