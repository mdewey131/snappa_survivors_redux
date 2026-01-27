use avian2d::prelude::*;
use bevy::{ecs::system::SystemId, prelude::*};
use lightyear::prelude::*;
use rand::Rng;

use crate::shared::{
    GameMainChannel,
    game_kinds::*,
    game_object_spawning::{self, spawn_game_object},
    game_rules::GameRules,
    lobby::{ClientStartGameMessage, ServerStartLoadingGameMessage},
    players::*,
    states::{AppState, InGameState},
    stats::RawStatsList, //xp::add_level_manager},
    weapons::{WeaponKind, add_weapon_to_player},
};

pub struct ClientGameLoadingPlugin;

impl Plugin for ClientGameLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LoadingLevel),
            (
                (spawn_player_character /*add_level_manager*/,).run_if(is_single_player),
                tmp_move_to_game,
            ),
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

/// For now, loading does nothing because I don't want to figure it out. Let's just get to the game stuff
fn tmp_move_to_game(
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<InGameState>>,
) {
    app_state.set(AppState::InGame);
    game_state.set(InGameState::InGame);
}

/// Very tmp while I don't have a query anywhwere for user's character selection
fn spawn_player_character(mut commands: Commands, game_kinds: Res<CurrentGameKind>) {
    let mut rng = rand::rng();
    let pos = (rng.random_range(-50.0..50.0), rng.random_range(-50.0..50.0));
    let player = Player {
        client: PeerId::Local(0),
    };

    let p_ent = spawn_game_object(
        &mut commands,
        game_kinds.0.unwrap(),
        Some(CharacterKind::Dewey),
        MultiPlayerComponentOptions::from(player),
        (player, Position(Vec2::new(pos.0, pos.1))),
    );

    add_weapon_to_player(
        p_ent,
        WeaponKind::DiceGuard,
        &mut commands,
        game_kinds.0.unwrap(),
    );
}
