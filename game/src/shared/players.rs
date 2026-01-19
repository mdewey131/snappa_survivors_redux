use crate::{
    shared::{
        colliders::{ColliderTypes, CommonColliderBundle},
        game_kinds::MultiPlayerComponentOptions,
        inputs::Movement,
    },
    utils::AssetFolder,
};
use avian2d::prelude::*;
use bevy::{ecs::query::QueryFilter, prelude::*};
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

impl From<Player> for MultiPlayerComponentOptions {
    fn from(value: Player) -> Self {
        Self {
            pred: true,
            interp: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Reflect)]
pub enum CharacterKind {
    #[default]
    Dewey,
}

impl From<CharacterKind> for AssetFolder {
    fn from(value: CharacterKind) -> Self {
        let s = match value {
            CharacterKind::Dewey => "survivors/dewey".into(),
        };
        Self(s)
    }
}

pub struct PlayerProtocolPlugin;
impl Plugin for PlayerProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Player>();
    }
}

fn shared_player_movement(mut velo: Mut<LinearVelocity>, input: Vec2) {
    const MS: f32 = 30.0;
    velo.0 = input * MS
}

pub fn player_movement<QF: QueryFilter>(
    q_mv_action: Query<(&ActionValue, &ActionOf<Player>), With<Action<Movement>>>,
    mut q_lv: Query<&mut LinearVelocity, (QF, With<Player>)>,
) {
    for (val, a_of) in &q_mv_action {
        if let Ok(mut lv) = q_lv.get_mut(a_of.entity()) {
            shared_player_movement(lv, val.as_axis2d());
        }
    }
}
