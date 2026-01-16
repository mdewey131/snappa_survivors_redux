use bevy::prelude::*;

mod dice_guard;
use dice_guard::ClientDiceGuardPlugin;

pub struct ClientWeaponsPlugin;
impl Plugin for ClientWeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientDiceGuardPlugin);
    }
}
