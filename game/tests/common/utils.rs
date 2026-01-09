use bevy::{prelude::*, time::TimeUpdateStrategy};
use snappa_survivors::build::{build_game_client_app, build_game_server_app};
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
