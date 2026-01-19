use bevy::prelude::*;
use lightyear::prelude::*;

use crate::{
    render::weapons::add_dice_guard_rendering_components,
    shared::{game_kinds::DefaultServerFilter, weapons::*},
};

pub struct DedicatedServerDiceGuardPlugin;
impl Plugin for DedicatedServerDiceGuardPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(dice_guard_activate::<With<Replicate>>)
            .add_observer(dice_guard_deactivate::<With<Replicate>>);
    }
}

pub struct DedicatedServerDiceGuardRenderPlugin;
impl Plugin for DedicatedServerDiceGuardRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(add_dice_guard_rendering_components::<DefaultServerFilter>);
    }
}
