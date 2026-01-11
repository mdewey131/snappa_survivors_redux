use crate::shared::{
    colliders::{ColliderTypes, CommonColliderBundle},
    inputs::Movement,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use lightyear::prelude::{AppComponentExt, PeerId};
use serde::{Deserialize, Serialize};

/// The component that describes a player.
/// This holds a record of the peer id so that,
/// if a client disconnects, we can still maintain
/// state of the character while we wait for that person
/// to come back
#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Reflect)]
pub struct Player {
    pub client: PeerId,
}

impl From<Player> for CommonColliderBundle {
    fn from(value: Player) -> Self {
        Self::new(
            RigidBody::Dynamic,
            Collider::capsule(20.0, 30.0),
            1.0,
            [ColliderTypes::Player].into(),
            [
                ColliderTypes::Enemy,
                ColliderTypes::StaticPickup,
                ColliderTypes::RemotePickup,
            ]
            .into(),
        )
    }
}

pub struct PlayerProtocolPlugin;
impl Plugin for PlayerProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Player>();
    }
}

fn shared_player_movement(mut velo: Mut<LinearVelocity>, input: Vec2) {
    const MS: f32 = 20.0;
    velo.0 = input * MS
}

pub fn player_movement<C: Component>(
    t: On<Fire<Movement>>,
    mut q_lv: Query<&mut LinearVelocity, With<C>>,
) {
    if let Ok(mut lv) = q_lv.get_mut(t.context) {
        shared_player_movement(lv, t.value);
    }
}
