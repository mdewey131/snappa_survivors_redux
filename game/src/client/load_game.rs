use avian2d::prelude::Position;
use bevy::prelude::*;

use crate::shared::{
    game_kinds::*,
    game_rules::{GameRules, MapKind},
    loading::{LevelLoadingState, spawn_player_character},
    pickups::HealthPickup,
    states::{AppState, InGameState, set_app_state_in_game},
    stats::xp::add_xp_manager,
};

#[cfg(feature = "dev")]
use crate::utils::zoo::*;

pub struct ClientGameLoadingPlugin;

impl Plugin for ClientGameLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(LevelLoadingState::LevelReady),
            ((assemble_level), set_app_state_in_game).run_if(in_state(AppState::LoadingLevel)),
        );
    }
}

pub fn client_transition_to_loading_state(
    arg_in: In<GameRules>,
    mut rules: ResMut<GameRules>,
    mut state: ResMut<NextState<AppState>>,
) {
    *rules = *arg_in;
    state.set(AppState::LoadingLevel)
}

fn assemble_level(mut commands: Commands, game_kinds: Res<CurrentGameKind>, rules: Res<GameRules>) {
    match game_kinds.0.unwrap() {
        GameKinds::SinglePlayer => match rules.map_type {
            #[cfg(feature = "dev")]
            MapKind::DevZoo => {
                info!("Loading Dev Zoo");
                let spawn_zoo_player_dummies = commands.register_system(spawn_zoo_characters);
                let spawn_zoo_weapons = commands.register_system(spawn_zoo_weapons);
                commands.run_system(spawn_zoo_player_dummies);
                commands.run_system(spawn_zoo_weapons);
            }
            MapKind::TheGreens => {
                info!("Loading the Greens");
                let spawn_pickup = commands.register_system(tmp_spawn_health_pickup);
                commands.run_system(spawn_pickup);

                let player_character_spawn_sys = commands.register_system(spawn_player_character);
                commands.run_system(player_character_spawn_sys);
                let level_manager_sys = commands.register_system(add_xp_manager);
                commands.run_system(level_manager_sys);
            }
            _ => {}
        },
        GameKinds::MultiPlayer => {}
    }
}

fn tmp_spawn_health_pickup(mut commands: Commands) {
    commands.spawn((
        HealthPickup { amount: 5.0 },
        Position(Vec2::new(500.0, 400.0)),
    ));
}
