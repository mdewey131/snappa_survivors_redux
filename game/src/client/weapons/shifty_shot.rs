use crate::shared::{
    combat::CombatSystemSet,
    game_kinds::{DefaultClientFilter, is_single_player},
    states::InGameState,
    weapons::shifty_shot::*,
};
use bevy::prelude::*;

pub struct ClientShiftyShotPlugin;
impl Plugin for ClientShiftyShotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            update_shifty_shot_attack::<DefaultClientFilter>
                .run_if(in_state(InGameState::InGame).and_then(is_single_player))
                .in_set(CombatSystemSet::Combat),
        )
        .add_observer(shifty_shot_activate::<DefaultClientFilter>)
        .add_observer(shifty_shot_deactivate::<DefaultClientFilter>);
    }
}

pub struct ClientShiftyShotRenderPlugin;
impl Plugin for ClientShiftyShotRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (reduce_orphaned_attack, restore_attack_size_on_target_found)
                .run_if(in_state(InGameState::InGame)),
        )
        .add_observer(add_shifty_shot_attack_sprite::<DefaultClientFilter>);
    }
}
