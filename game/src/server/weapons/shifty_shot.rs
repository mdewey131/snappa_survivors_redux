use bevy::prelude::*;

use crate::shared::{
    combat::CombatSystemSet, game_kinds::DefaultServerFilter, states::InGameState,
    weapons::shifty_shot::*,
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

pub struct DedicatedServerShiftyShotRenderPlugin;
impl Plugin for DedicatedServerShiftyShotRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (reduce_orphaned_attack, restore_attack_size_on_target_found)
                .run_if(in_state(InGameState::InGame)),
        )
        .add_observer(add_shifty_shot_attack_sprite::<DefaultServerFilter>);
    }
}
