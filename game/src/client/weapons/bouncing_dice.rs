use bevy::prelude::*;

use crate::{
    render::weapons::*,
    shared::{game_kinds::DefaultClientFilter, weapons::*},
};
pub struct ClientBouncingDicePlugin;
impl Plugin for ClientBouncingDicePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, bouncing_dice_attack::<DefaultClientFilter>)
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
