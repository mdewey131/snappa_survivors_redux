use bevy::prelude::*;
use lightyear::prelude::{AppComponentExt, PeerId};
use serde::{Deserialize, Serialize};

/// The component that describes a player.
/// This holds a record of the peer id so that,
/// if a client disconnects, we can still maintain
/// state of the character while we wait for that person
/// to come back
#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub struct Player {
    pub client: PeerId,
}

pub struct PlayerProtocolPlugin;
impl Plugin for PlayerProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Player>();
    }
}
