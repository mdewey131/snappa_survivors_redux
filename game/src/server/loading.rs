use crate::shared::{
    players::Player,
    states::*,
    weapons::{WeaponKind, add_weapon_to_player},
};
use avian2d::prelude::Position;
use bevy::prelude::*;
use lightyear::prelude::{
    Client, ControlledBy, Lifetime, LinkOf, NetworkTarget, PredictionTarget, RemoteId, Replicate,
};
use rand::Rng;

pub struct DedicatedServerLoadingPlugin;

impl Plugin for DedicatedServerLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LoadingLevel),
            (spawn_player_characters, tmp_move_to_game).chain(),
        );
    }
}

fn spawn_player_characters(
    mut commands: Commands,
    q_clients: Query<(Entity, &RemoteId), With<LinkOf>>,
) {
    for (ent, remote) in &q_clients {
        let mut rng = rand::rng();
        let pos = (rng.random_range(-50.0..50.0), rng.random_range(-50.0..50.0));
        let player = commands
            .spawn((
                Player { client: remote.0 },
                Position(Vec2::new(pos.0, pos.1)),
                Replicate::to_clients(NetworkTarget::All),
                PredictionTarget::to_clients(NetworkTarget::All),
                ControlledBy {
                    owner: ent,
                    lifetime: Lifetime::default(),
                },
            ))
            .id();

        add_weapon_to_player(player, WeaponKind::DiceGuard, &mut commands);
    }
}

fn tmp_move_to_game(
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<InGameState>>,
) {
    app_state.set(AppState::InGame);
    game_state.set(InGameState::InGame);
}
