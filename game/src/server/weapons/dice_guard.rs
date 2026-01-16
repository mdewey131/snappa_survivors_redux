use bevy::prelude::*;
use lightyear::prelude::*;

use crate::shared::weapons::dice_guard::dice_guard_activate;

pub struct DedicatedServerDiceGuardPlugin;
impl Plugin for DedicatedServerDiceGuardPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(dice_guard_activate::<With<Replicate>>);
    }
}
