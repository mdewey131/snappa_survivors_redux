use bevy::prelude::*;
use clap::Parser;
use snappa_survivors::build::Cli;

fn main() {
    let cli = Cli::parse();
    let mut app = App::new();

    cli.build_game_app(&mut app);

    app.run();
}
