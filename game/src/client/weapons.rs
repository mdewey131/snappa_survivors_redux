use bevy::prelude::*;

mod bouncing_dice;
mod dice_guard;
mod throw_hands;

pub use bouncing_dice::*;
pub use dice_guard::*;
pub use throw_hands::*;

pub struct ClientWeaponsPlugin;
impl Plugin for ClientWeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ClientBouncingDicePlugin,
            ClientDiceGuardPlugin,
            ClientThrowHandsPlugin,
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
        ));
    }
}
