use crate::shared::{
    combat::CharacterFacing,
    game_kinds::{CurrentGameKind, MultiPlayerComponentOptions},
    game_object_spawning::spawn_game_object,
    game_rules::GameRules,
    loading::spawn_characters_in_multiplayer,
    players::{CharacterKind, Player, PlayerBaseBundle, PlayerWeapons},
    states::*,
    stats::xp::add_level_manager,
    upgrades::PlayerUpgradeSlots,
    weapons::{WeaponKind, add_weapon_to_character},
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
            (
                add_level_manager,
                spawn_characters_in_multiplayer,
                tmp_move_to_game,
            )
                .chain(),
        );
    }
}

fn load_level(mut commands: Commands, game_rules: Res<GameRules>) {}

fn tmp_move_to_game(
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<InGameState>>,
) {
    app_state.set(AppState::InGame);
    game_state.set(InGameState::InGame);
}
