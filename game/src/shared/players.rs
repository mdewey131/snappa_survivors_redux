
use crate::{
    shared::{
        colliders::{ColliderTypes, CommonColliderBundle, RecentlyCollided},
        combat::{CharacterFacing, CombatEntityActive},
        damage::Dead,
        game_kinds::{MultiPlayerComponentOptions, SinglePlayer},
        inputs::Movement,
        stats::{
            StatKind, StatList,
            components::{Health, MovementSpeed, PickupRadius},
        },
        upgrades::PlayerUpgradeSlots,
        weapons::WeaponKind,
    },
    utils::AssetFolder,
};
use avian2d::prelude::*;
use bevy::{
    ecs::{entity_disabling::Disabled, query::QueryFilter},
    platform::collections::HashMap,
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

mod spawning;
pub use spawning::*;

/// The component that describes a player.
/// This holds a record of the peer id so that,
/// if a client disconnects, we can still maintain
/// state of the character while we wait for that person
/// to come back
#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Reflect)]
pub struct Player {
    pub client: Option<PeerId>,
    pub character: CharacterKind,
}

impl From<Player> for CommonColliderBundle {
    fn from(_value: Player) -> Self {
        Self::new(
            RigidBody::Dynamic,
            Collider::capsule(20.0, 30.0),
            1.0,
            [ColliderTypes::Player].into(),
            [
                ColliderTypes::Enemy,
                ColliderTypes::StaticPickup,
                ColliderTypes::RemotePickup,
                ColliderTypes::SolidObject,
            ]
            .into(),
        )
    }
}

impl From<Player> for MultiPlayerComponentOptions {
    fn from(_value: Player) -> Self {
        Self {
            pred: true,
            interp: false,
        }
    }
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Reflect, Serialize, Deserialize, EnumIter, Hash, Eq,
)]
pub enum CharacterKind {
    #[default]
    Dewey,
    Matthew,
    Paul,
    Shaunt,
    Mark,
    Ryan,
    Gabe,
    Finn,
}

impl From<CharacterKind> for String {
    fn from(value: CharacterKind) -> Self {
        
        match value {
            CharacterKind::Dewey => "Dewey".into(),
            CharacterKind::Finn => "Finn".into(),
            CharacterKind::Gabe => "Gabe".into(),
            CharacterKind::Mark => "Mark".into(),
            CharacterKind::Matthew => "Matthew".into(),
            CharacterKind::Paul => "Paul".into(),
            CharacterKind::Ryan => "Ryan".into(),
            CharacterKind::Shaunt => "Shaunt".into(),
        }
    }
}

/*
impl From<CharacterKind> for Path {
    fn from(value: CharacterKind) -> Self {
        let s: String = match value {
            CharacterKind::Dewey => "survivors/dewey".into(),
            CharacterKind::Finn => "survivors/finn".into(),
            CharacterKind::Gabe => "survivors/gabe".into(),
            CharacterKind::Mark => "survivors/mark".into(),
            CharacterKind::Matthew => "survivors/matthew".into(),
            CharacterKind::Paul => "survivors/paul".into(),
            CharacterKind::Ryan => "survivors/ryan".into(),
            CharacterKind::Shaunt => "survivors/shaunt".into(),
        };
        Self::new(&s)
    }
}
*/

impl From<CharacterKind> for AssetFolder {
    fn from(value: CharacterKind) -> Self {
        let s = match value {
            CharacterKind::Dewey => "survivors/dewey".into(),
            CharacterKind::Finn => "survivors/finn".into(),
            CharacterKind::Gabe => "survivors/gabe".into(),
            CharacterKind::Mark => "survivors/mark".into(),
            CharacterKind::Matthew => "survivors/matthew".into(),
            CharacterKind::Paul => "survivors/paul".into(),
            CharacterKind::Ryan => "survivors/ryan".into(),
            CharacterKind::Shaunt => "survivors/shaunt".into(),
        };
        Self(s)
    }
}

impl CharacterKind {
    pub fn starting_weapon(&self) -> WeaponKind {
        match self {
            CharacterKind::Dewey => WeaponKind::ShiftyShot,
            CharacterKind::Finn => WeaponKind::DiceGuard,
            CharacterKind::Gabe => WeaponKind::DiceGuard,
            CharacterKind::Mark => WeaponKind::FlurryOfBlows,
            CharacterKind::Matthew => WeaponKind::BouncingDice,
            CharacterKind::Paul => WeaponKind::PaddleBack,
            CharacterKind::Ryan => WeaponKind::ThrowHands,
            CharacterKind::Shaunt => WeaponKind::ThrowHands,
        }
    }
}

#[derive(Component, Debug, Reflect, Default)]
pub struct PlayerWeapons(pub HashMap<WeaponKind, Entity>);

/// Placed on a player entity at initialization.
///
/// Because we want to insert certain elements of a
/// player in each environment (client, server),
/// this bundle is just the minimum set of things that
/// we know we can attach  at spawn
/// (generally, that's things that can be networked)
#[derive(Bundle)]
pub struct PlayerBaseBundle {
    pub player: Player,
    pub position: Position,
    pub upgrade_slots: PlayerUpgradeSlots,
    pub weapons: PlayerWeapons,
    pub facing: CharacterFacing,
}

#[derive(Component, Debug)]
pub struct Reviving(pub Timer);

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
    mut q_lv: Query<(&MovementSpeed, &mut LinearVelocity), (QF, With<Player>, CombatEntityActive)>,
) {
    for (val, a_of) in &q_mv_action {
        if let Ok((ms, lv)) = q_lv.get_mut(a_of.entity()) {
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

/// Need to do a variety of things on player death
///
/// pull stats to remove revives,
pub fn on_player_death(
    mut commands: Commands,
    mut q_player: Query<
        (Entity, &mut LinearVelocity, &Children, &mut StatList),
        (With<Player>, Added<Dead>),
    >,
) {
    for (ent, mut velo, children, mut list) in &mut q_player {
        velo.0 = Vec2::ZERO;
        commands.entity(ent).insert(ColliderDisabled);
        for child in children {
            commands.entity(*child).insert(Disabled);
        }
        let stat = list.remove(&StatKind::Revive);
        if let Some(mut s) = stat {
            let rev_val = s.get_current().unwrap_or(0.0 - f32::EPSILON);
            if rev_val > 0.0 {
                s.base_value -= 1.0;
                list.list.insert(StatKind::Revive, s);
                commands
                    .entity(ent)
                    .remove::<Dead>()
                    .insert(Reviving(Timer::from_seconds(1.0, TimerMode::Once)));
            }
        } else {
            commands.entity(ent).insert(Text2d::from("DEAD"));
        }
    }
}

pub fn on_player_revive(
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    mut q_player: Query<(Entity, &mut Reviving, &Children, &mut Health), With<Player>>,
) {
    for (player, mut rev, children, mut health) in &mut q_player {
        rev.0.tick(time.delta());
        if rev.0.is_finished() {
            commands
                .entity(player)
                .remove::<Reviving>()
                .remove::<ColliderDisabled>();

            health.current = health.max() * 0.5;
            for child in children {
                commands.entity(*child).remove::<Disabled>();
            }
        }
    }
}

pub fn update_player_facing_direction<QF: QueryFilter>(
    mut q_player: Query<(&mut CharacterFacing, &Actions<Player>), QF>,
    q_movement: Query<&ActionValue, With<Action<Movement>>>,
) {
    for (mut facing, actions) in &mut q_player {
        for a_ent in actions.iter() {
            if let Ok(a_val) = q_movement.get(a_ent) {
                let dir = a_val.as_axis2d();
                let next = facing.next_direction(dir);
                let current = facing.c_dir;
                if current != next {
                    facing.c_dir = next;
                }
            }
        }
    }
}
