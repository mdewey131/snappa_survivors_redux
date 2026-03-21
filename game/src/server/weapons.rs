use bevy::prelude::*;

mod bouncing_dice;
mod bumpin_tunes;
mod dice_guard;
mod throw_hands;

pub use bouncing_dice::*;
pub use bumpin_tunes::*;
pub use dice_guard::*;
pub use throw_hands::*;

pub struct DedicatedServerWeaponsPlugin;
impl Plugin for DedicatedServerWeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DedicatedServerBumpinTunesPlugin,
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
