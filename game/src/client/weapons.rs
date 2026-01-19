use bevy::prelude::*;

mod dice_guard;
pub use dice_guard::*;

pub struct ClientWeaponsPlugin;
impl Plugin for ClientWeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientDiceGuardPlugin);
    }
}
