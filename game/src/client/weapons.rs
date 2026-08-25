use bevy::prelude::*;

mod bouncing_dice;
mod bumpin_tunes;
mod dice_guard;
mod shifty_shot;
mod throw_hands;

pub use bouncing_dice::*;
pub use bumpin_tunes::*;
pub use dice_guard::*;
pub use shifty_shot::*;
pub use throw_hands::*;

pub struct ClientWeaponsPlugin;
impl Plugin for ClientWeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ClientBouncingDicePlugin,
            ClientBumpinTunesPlugin,
            //ClientDiceGuardPlugin,
            ClientThrowHandsPlugin,
            ClientShiftyShotPlugin,
        ));
    }
}

pub struct ClientWeaponsRenderPlugin;
impl Plugin for ClientWeaponsRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ClientBouncingDiceRenderPlugin,
            ClientDiceGuardRenderPlugin,
            ClientThrowHandsRenderPlugin,
            ClientShiftyShotRenderPlugin,
        ));
    }
}
