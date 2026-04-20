use crate::shared::{
    combat::CharacterFacing,
    game_kinds::{CurrentGameKind, MultiPlayerComponentOptions},
    game_object_spawning::spawn_game_object,
    game_rules::GameRules,
    loading::LevelLoadingState,
    maps::*,
    players::{CharacterKind, Player, PlayerBaseBundle, PlayerWeapons},
    states::*,
    stats::xp::add_xp_manager,
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
        app.add_systems(OnEnter(AppState::LoadingLevel), add_map_loading_systems)
            .add_systems(
                OnEnter(LevelLoadingState::LevelLoading),
                run_map_loading_systems,
            )
            .add_systems(
                OnEnter(LevelLoadingState::LevelReady),
                (add_xp_manager, set_app_state_in_game)
                    .chain()
                    .run_if(in_state(AppState::LoadingLevel)),
            );
    }
}

fn load_level(mut commands: Commands, game_rules: Res<GameRules>) {}
