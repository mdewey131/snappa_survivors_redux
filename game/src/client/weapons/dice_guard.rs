use bevy::prelude::*;

use crate::shared::{game_kinds::DefaultClientFilter, weapons::dice_guard::dice_guard_activate};

pub struct ClientDiceGuardPlugin;
impl Plugin for ClientDiceGuardPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(dice_guard_activate::<DefaultClientFilter>);
    }
}
