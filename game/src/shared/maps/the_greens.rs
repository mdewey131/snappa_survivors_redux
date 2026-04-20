use crate::{
    shared::{game_kinds::CurrentGameKind, lobby::PlayerInLobby, players::*},
    utils::SpawnPattern,
};
use bevy::prelude::*;
use lightyear::prelude::*;

pub fn spawn_characters_the_greens(
    mut commands: Commands,
    game_kinds: Res<CurrentGameKind>,
    q_player: Query<(Entity, &PlayerInLobby, Option<&RemoteId>)>,
) {
    let n_char = q_player.iter().len();
    let mut spawn_pos = SpawnPattern::Circle {
        amount: n_char as u8,
        center: Vec2::ZERO,
        radius: 500.0,
        radius_only: true,
    }
    .to_positions();

    spawn_characters(&mut spawn_pos, &mut commands, &game_kinds, &q_player);
}

pub fn spawn_interactables_the_greens() {}

pub fn enemy_spawners_the_greens() {}

pub fn map_elements_the_greens() {}
