use avian2d::prelude::*;
use bevy::{ecs::system::SystemId, prelude::*};
use lightyear::prelude::*;
use rand::Rng;

use crate::shared::{
    GameMainChannel,
    game_kinds::{SinglePlayer, is_single_player},
    game_rules::GameRules,
    lobby::{ClientStartGameMessage, ServerStartLoadingGameMessage},
    players::*,
    states::{AppState, InGameState},
    weapons::{WeaponKind, add_weapon_to_player},
};

pub struct ClientGameLoadingPlugin;

impl Plugin for ClientGameLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LoadingLevel),
            (
                spawn_player_character.run_if(is_single_player),
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
fn spawn_player_character(mut commands: Commands) {
    let mut rng = rand::rng();
    let pos = (rng.random_range(-50.0..50.0), rng.random_range(-50.0..50.0));
    let player = commands
        .spawn((
            Player {
                client: PeerId::Local(0),
            },
            Position(Vec2::new(pos.0, pos.1)),
            SinglePlayer,
        ))
        .id();
    add_weapon_to_player(player, WeaponKind::DiceGuard, &mut commands);
}
