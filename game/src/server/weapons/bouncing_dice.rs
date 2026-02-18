use bevy::prelude::*;
use lightyear::prelude::*;

use crate::{
    render::weapons::add_bouncing_dice_rendering_components,
    shared::{game_kinds::DefaultServerFilter, weapons::*},
};
pub struct DedicatedServerBouncingDicePlugin;
impl Plugin for DedicatedServerBouncingDicePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, bouncing_dice_attack::<DefaultServerFilter>)
            .add_observer(bouncing_dice::on_activate::<DefaultServerFilter>)
            .add_observer(bouncing_dice::on_deactivate::<DefaultServerFilter>);
    }
}

pub struct DedicatedServerBouncingDiceRenderPlugin;
impl Plugin for DedicatedServerBouncingDiceRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(add_bouncing_dice_rendering_components::<DefaultServerFilter>);
    }
}
