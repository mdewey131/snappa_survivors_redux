use crate::shared::{
    game_kinds::{CurrentGameKind, MultiPlayerComponentOptions},
    game_object_spawning::spawn_game_object,
    players::{CharacterKind, Player, PlayerBaseBundle, PlayerWeapons},
    states::*,
    stats::xp::add_level_manager,
    upgrades::PlayerUpgradeSlots,
    weapons::{WeaponKind, add_weapon_to_player},
};
use avian2d::prelude::Position;
use bevy::prelude::*;
use lightyear::prelude::{ControlledBy, Lifetime, LinkOf, RemoteId};
use rand::Rng;

pub struct DedicatedServerLoadingPlugin;

impl Plugin for DedicatedServerLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LoadingLevel),
            (add_level_manager, spawn_player_characters, tmp_move_to_game).chain(),
        );
    }
}

fn spawn_player_characters(
    mut commands: Commands,
    game_kinds: Res<CurrentGameKind>,
    q_clients: Query<(Entity, &RemoteId), With<LinkOf>>,
) {
    for (ent, remote) in &q_clients {
        let mut rng = rand::rng();
        let pos = (rng.random_range(-50.0..50.0), rng.random_range(-50.0..50.0));

        let player = Player { client: remote.0 };

        let p_ent = spawn_game_object(
            &mut commands,
            game_kinds.0.unwrap(),
            Some(CharacterKind::Dewey),
            MultiPlayerComponentOptions::from(player),
            (
                PlayerBaseBundle {
                    player,
                    position: Position(Vec2::new(pos.0, pos.1)),
                    upgrade_slots: PlayerUpgradeSlots::new(5, 5),
                    weapons: PlayerWeapons::default(),
                },
                ControlledBy {
                    owner: ent,
                    lifetime: Lifetime::default(),
                },
            ),
        );

        add_weapon_to_player(
            p_ent,
            WeaponKind::ThrowHands,
            &mut commands,
            game_kinds.0.unwrap(),
        );
    }
}

fn tmp_move_to_game(
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<InGameState>>,
) {
    app_state.set(AppState::InGame);
    game_state.set(InGameState::InGame);
}
