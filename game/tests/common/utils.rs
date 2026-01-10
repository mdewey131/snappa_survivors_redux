use bevy::{prelude::*, time::TimeUpdateStrategy};
use lightyear::prelude::Connect;
use snappa_survivors::{
    build::{build_game_client_app, build_game_server_app},
    client::game_client::GameClient,
    shared::{game_kinds::CurrentGameKind, states::AppState},
};
use std::time::Duration;

pub fn setup_test_client() -> App {
    let mut app = App::new();
    build_game_client_app(&mut app, None, false);
    app
}

pub fn setup_dedicated_server() -> App {
    let mut app = App::new();
    build_game_server_app(&mut app, false);
    app
}

pub fn tick_app(app: &mut App, timestep: f64) {
    let strategy = TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(timestep));

    if let Some(mut update_strategy) = app.world_mut().get_resource_mut::<TimeUpdateStrategy>() {
        *update_strategy = strategy;
    } else {
        app.insert_resource(strategy);
    }

    app.update();
}

pub fn move_to_single_player(app: &mut App) {
    let sys = app.register_system(snappa_survivors::client::transition_to_single_player);

    app.world_mut()
        .run_system(sys)
        .expect("Failure to transition to single player");
}

/// Returns in the order (Server, Client)
pub fn setup_multiplayer_connected_apps() -> (App, App) {
    // Spawn a server and the client on this app
    let mut client_app = setup_test_client();
    let mut server_app = setup_dedicated_server();

    client_app.update();
    server_app.update();
    // This works for now, but you probably will want a way to configure other settings in the future so that this can work more better
    client_app.world_mut().spawn(GameClient::SINGLE_PLAYER);

    // TODO This should be consolidated into one function when we do multiplayer connection. For later
    let mut game_kind = client_app.world_mut().resource_mut::<CurrentGameKind>();
    game_kind.0 = Some(snappa_survivors::shared::game_kinds::GameKinds::MultiPlayer);
    let mut c_state = client_app.world_mut().resource_mut::<NextState<AppState>>();
    c_state.set(AppState::EstablishServerConnection);

    for _update in (0..30) {
        tick_app(&mut client_app, 1.0 / 64.0);
        tick_app(&mut server_app, 1.0 / 64.0);
    }
    (server_app, client_app)
}
