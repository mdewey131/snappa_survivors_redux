use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use lightyear::prelude::{Controlled, Predicted, Replicate};

use crate::{
    render::player::rendering_on_player_add,
    shared::{
        colliders::CommonColliderBundle,
        combat::CombatSystemSet,
        game_kinds::{DefaultClientFilter, SinglePlayer},
        inputs::Movement,
        players::*,
        states::InGameState,
    },
};

pub struct ClientPlayerPlugin;

impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                player_movement::<DefaultClientFilter>,
                update_player_facing_direction::<DefaultClientFilter>,
            )
                .in_set(CombatSystemSet::Combat)
                .run_if(in_state(InGameState::InGame)),
        )
        .add_observer(add_non_networked_player_components::<DefaultClientFilter>);
    }
}

pub struct ClientPlayerRenderPlugin;
impl Plugin for ClientPlayerRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, rendering_on_player_add::<DefaultClientFilter>);
    }
}
