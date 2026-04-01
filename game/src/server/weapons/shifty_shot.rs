use bevy::prelude::*;

use crate::{
    render::weapons::*,
    shared::{
        combat::CombatSystemSet,
        game_kinds::{DefaultServerFilter, is_single_player},
        states::InGameState,
        weapons::shifty_shot::*,
    },
};

pub struct DedicatedServerShiftyShotPlugin;
impl Plugin for DedicatedServerShiftyShotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            update_shifty_shot_attack::<DefaultServerFilter>
                .run_if(in_state(InGameState::InGame))
                .in_set(CombatSystemSet::Combat),
        )
        .add_observer(shifty_shot_activate::<DefaultServerFilter>)
        .add_observer(shifty_shot_deactivate::<DefaultServerFilter>);
    }
}

/*
pub struct DedicatedServerThrowHandsRenderPlugin;
impl Plugin for DedicatedServerThrowHandsRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(add_throwing_hands_attack_sprite::<DefaultServerFilter>)
            .add_systems(Update, throwing_hands_sprite_follow::<DefaultServerFilter>);
    }
}
 */
