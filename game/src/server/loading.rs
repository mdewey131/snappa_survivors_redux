use crate::shared::{players::Player, states::*};
use avian2d::prelude::Position;
use bevy::prelude::*;
use lightyear::prelude::{Client, LinkOf, NetworkTarget, PredictionTarget, RemoteId, Replicate};
use rand::Rng;

pub struct ServerLoadingPlugin;

impl Plugin for ServerLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LoadingLevel),
            (spawn_player_characters, tmp_move_to_game).chain(),
        );
    }
}

fn spawn_player_characters(mut commands: Commands, q_clients: Query<(&RemoteId), With<LinkOf>>) {
    for remote in &q_clients {
        let mut rng = rand::rng();
        let pos = (rng.random_range(-50.0..50.0), rng.random_range(-50.0..50.0));
        commands.spawn((
            Player { client: remote.0 },
            Position(Vec2::new(pos.0, pos.1)),
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::All),
        ));
    }
}

fn tmp_move_to_game(
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<InGameState>>,
) {
    app_state.set(AppState::InGame);
    game_state.set(InGameState::InGame);
}
