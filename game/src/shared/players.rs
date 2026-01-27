use crate::{
    shared::{
        colliders::{ColliderTypes, CommonColliderBundle, RecentlyCollided},
        damage::Dead,
        game_kinds::{CurrentGameKind, MultiPlayerComponentOptions, SinglePlayer},
        inputs::Movement,
        stats::components::{MovementSpeed, PickupRadius},
    },
    utils::AssetFolder,
};
use avian2d::prelude::*;
use bevy::{ecs::query::QueryFilter, prelude::*};
use bevy_enhanced_input::prelude::*;
use lightyear::prelude::*;
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

/// Marker component for the pickup radius that a player has
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerPickupRadius;

pub struct PlayerProtocolPlugin;
impl Plugin for PlayerProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Player>();
    }
}

fn shared_player_movement(mut velo: Mut<LinearVelocity>, ms: f32, input: Vec2) {
    velo.0 = input.normalize_or_zero() * ms
}

pub fn player_movement<QF: QueryFilter>(
    q_mv_action: Query<(&ActionValue, &ActionOf<Player>), With<Action<Movement>>>,
    mut q_lv: Query<(&MovementSpeed, &mut LinearVelocity), (QF, With<Player>, Without<Dead>)>,
) {
    for (val, a_of) in &q_mv_action {
        if let Ok((ms, mut lv)) = q_lv.get_mut(a_of.entity()) {
            shared_player_movement(lv, ms.current, val.as_axis2d());
        }
    }
}

pub fn add_non_networked_player_components<QF: QueryFilter>(
    trigger: On<Add, Player>,
    mut commands: Commands,
    q_pred: Query<(Has<Controlled>, Has<SinglePlayer>, &Player, &PickupRadius), QF>,
) {
    if let Ok((cont, sp, p, pur)) = q_pred.get(trigger.entity) {
        if cont || sp {
            commands.spawn((
                ActionOf::<Player>::new(trigger.entity),
                Action::<Movement>::new(),
                Bindings::spawn(Cardinal::wasd_keys()),
                // This isn't in the example, but
                // it seems that you need this so that the
                // replication works in a single player scenario. It doesn't appear
                // to affect MP too much
                Replicate::to_server(),
            ));
        }
        // regardless, add the collider components
        commands
            .entity(trigger.entity)
            .insert((
                CommonColliderBundle::from(*p),
                Name::from("Player"),
                RecentlyCollided::default(),
            ))
            .with_child((
                Collider::circle(pur.0),
                Sensor,
                PlayerPickupRadius,
                CollisionLayers::new(
                    [ColliderTypes::PlayerPickupRadius],
                    [ColliderTypes::RemotePickup],
                ),
            ));
    }
}

// On death, we want to spawn a region that can be used for players to revive their friend
/*
pub fn on_death(
    trigger: On<Add, Dead>,
    gk: Res<CurrentGameKind>,
    q_player: Query<(), With<Player>>,
) {
}
 */
