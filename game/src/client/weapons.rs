use bevy::prelude::*;

mod dice_guard;
mod throw_hands;
pub use dice_guard::*;
pub use throw_hands::*;

pub struct ClientWeaponsPlugin;
impl Plugin for ClientWeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ClientDiceGuardPlugin, ClientThrowHandsPlugin));
    }
}
