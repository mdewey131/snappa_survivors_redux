use bevy::prelude::*;
use lightyear::prelude::Replicate;

use crate::{
    render::player::rendering_on_player_add,
    shared::{
        colliders::CommonColliderBundle, combat::CombatSystemSet, game_kinds::DefaultServerFilter,
        players::*,
    },
};

pub struct ServerPlayerPlugin;

impl Plugin for ServerPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (player_movement::<With<Replicate>>).in_set(CombatSystemSet::Combat),
        )
        .add_observer(add_non_networked_player_components::<DefaultServerFilter>);
    }
}

pub struct ServerPlayerRenderPlugin;
impl Plugin for ServerPlayerRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, rendering_on_player_add::<With<Replicate>>);
    }
}
