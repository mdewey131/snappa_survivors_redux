use bevy::prelude::*;

use crate::{
    render::weapons::add_dice_guard_rendering_components,
    shared::{
        game_kinds::DefaultClientFilter,
        stats::{StatKind, StatList, StatModifier, StatModifierMethod},
        weapons::*,
    },
};

pub struct ClientDiceGuardPlugin;
impl Plugin for ClientDiceGuardPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(dice_guard_activate::<DefaultClientFilter>)
            .add_observer(dice_guard_deactivate::<DefaultClientFilter>);
    }
}

pub struct ClientDiceGuardRenderPlugin;
impl Plugin for ClientDiceGuardRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(add_dice_guard_rendering_components::<DefaultClientFilter>);
    }
}
