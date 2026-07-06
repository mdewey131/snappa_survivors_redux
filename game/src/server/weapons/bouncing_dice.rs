use bevy::prelude::*;

use crate::{
    render::weapons::*,
    shared::{
        combat::CombatSystemSet, game_kinds::DefaultServerFilter, states::InGameState, weapons::*,
    },
};
pub struct DedicatedServerBouncingDicePlugin;
impl Plugin for DedicatedServerBouncingDicePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            bouncing_dice_attack::<DefaultServerFilter>
                .run_if(in_state(InGameState::InGame))
                .in_set(CombatSystemSet::Combat),
        )
        .add_observer(bouncing_dice::on_activate::<DefaultServerFilter>)
        .add_observer(bouncing_dice::on_deactivate::<DefaultServerFilter>);
    }
}

pub struct DedicatedServerBouncingDiceRenderPlugin;
impl Plugin for DedicatedServerBouncingDiceRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_bouncing_dice_render_components::<DefaultServerFilter>
                .run_if(in_state(InGameState::InGame))
                .in_set(CombatSystemSet::Combat),
        )
        .add_observer(add_bouncing_dice_rendering_components::<DefaultServerFilter>);
    }
}
