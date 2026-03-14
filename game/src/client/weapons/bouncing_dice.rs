use bevy::prelude::*;

use crate::{
    render::weapons::*,
    shared::{
        combat::CombatSystemSet, game_kinds::DefaultClientFilter, states::InGameState, weapons::*,
    },
};
pub struct ClientBouncingDicePlugin;
impl Plugin for ClientBouncingDicePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            bouncing_dice_attack::<DefaultClientFilter>
                .run_if(in_state(InGameState::InGame))
                .in_set(CombatSystemSet::Combat),
        )
        .add_observer(bouncing_dice::on_activate::<DefaultClientFilter>)
        .add_observer(bouncing_dice::on_deactivate::<DefaultClientFilter>);
    }
}

pub struct ClientBouncingDiceRenderPlugin;
impl Plugin for ClientBouncingDiceRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_bouncing_dice_render_components::<DefaultClientFilter>,
        )
        .add_observer(add_bouncing_dice_rendering_components::<DefaultClientFilter>);
    }
}
