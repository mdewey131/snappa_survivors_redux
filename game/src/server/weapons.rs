use crate::server::weapons::dice_guard::DedicatedServerDiceGuardPlugin;
use bevy::prelude::*;

mod dice_guard;

pub struct DedicatedServerWeaponsPlugin;
impl Plugin for DedicatedServerWeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DedicatedServerDiceGuardPlugin);
    }
}
