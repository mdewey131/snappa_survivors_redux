use bevy::prelude::*;

mod dice_guard;
pub use dice_guard::*;
mod throw_hands;
pub use throw_hands::*;

pub struct DedicatedServerWeaponsPlugin;
impl Plugin for DedicatedServerWeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DedicatedServerDiceGuardPlugin)
            .add_plugins(DedicatedServerThrowHandsPlugin);
    }
}
