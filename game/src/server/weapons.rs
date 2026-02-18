use bevy::prelude::*;

mod bouncing_dice;
pub use bouncing_dice::*;
mod dice_guard;
pub use dice_guard::*;
mod throw_hands;
pub use throw_hands::*;

pub struct DedicatedServerWeaponsPlugin;
impl Plugin for DedicatedServerWeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DedicatedServerDiceGuardPlugin,
            DedicatedServerThrowHandsPlugin,
            DedicatedServerBouncingDicePlugin,
        ));
    }
}

pub struct DedicatedServerWeaponsRenderPlugin;
impl Plugin for DedicatedServerWeaponsRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DedicatedServerDiceGuardRenderPlugin,
            DedicatedServerThrowHandsRenderPlugin,
            DedicatedServerBouncingDiceRenderPlugin,
        ));
    }
}
