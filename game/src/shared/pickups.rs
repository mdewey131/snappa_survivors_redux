use crate::shared::{
    colliders::*,
    combat::CombatSystemSet,
    states::InGameState,
    stats::{
        components::{Health, XPGain},
        xp::XPManager,
    },
};
use avian2d::prelude::*;
use bevy::{ecs::entity::MapEntities, prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

pub const HEALTH_PICKUP_SPAWNER_COOLDOWN: f32 = 30.0;

pub const XP_PICKUP_BASE_MOVE_SPEED: f32 = 5000.0;
pub const XP_PICKUP_CURVE_TIME_TO_ZERO: f32 = 0.25;

#[derive(Component, Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct HealthPickup {
    pub amount: f32,
}

#[derive(Component)]
#[require(Name = Name::from("Health Pickup Spawner"))]
pub struct HealthPickupSpawner {
    pub pickup: Entity,
    pub hp_amount: f32,
    pub timer: Timer,
}

#[derive(Component, Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct XPPickup {
    pub val: f32,
    pub targeting: Option<Entity>,
    pub t_time: f32,
}
impl XPPickup {
    pub fn new(v: f32) -> Self {
        Self {
            val: v,
            targeting: None,
            t_time: 0.0,
        }
    }
}
impl MapEntities for XPPickup {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        if self.targeting.is_some() {
            self.targeting = Some(entity_mapper.get_mapped(self.targeting.unwrap()))
        } else {
        }
    }
}

/// Marks the collision effect where the orb follows the player upon contact with their collision layer
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct XPPickupFollowPlayer;
impl CollisionEffect for XPPickupFollowPlayer {
    fn apply_to(&self, coms: &mut Commands, to: Entity, from: Entity) {
        coms.queue(move |world: &mut World| {
            // The colliding entity in this case is the child of the player, so we have to do this
            let parent = world.get::<ChildOf>(to);
            let p_ent = if let Some(p) = parent {
                p.0
            } else {
                return;
            };
            let mut ent_mut = world.get_entity_mut(from);
            if let Ok(ref mut em) = ent_mut {
                unsafe {
                    let xp_orb = em.get_components_mut_unchecked::<&mut XPPickup>();
                    if let Some(mut xp) = xp_orb {
                        xp.targeting = Some(p_ent)
                    }
                }
                em
                    // We don't want the ability to follow once we have started following
                    // This initially removed the collider. Don't do that, you won't be able to pick up the orb after
                    .remove::<AppliesCollisionEffect<XPPickupFollowPlayer>>();
            }
        });
    }
}

pub struct PickupsProtocolPlugin;
impl Plugin for PickupsProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<XPPickup>()
            .add_prediction()
            .add_map_entities();
    }
}

pub struct SharedPickupsPlugin;
impl Plugin for SharedPickupsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (xp_orb_update, hp_spawner_update)
                .in_set(CombatSystemSet::Combat)
                .run_if(in_state(InGameState::InGame)),
        )
        .add_systems(
            FixedPostUpdate,
            (
                apply_collision_effect_on_collision_start::<XPPickupFollowPlayer>,
                apply_collision_effect_on_collision_start::<TriggerPickup>,
            )
                .after(PhysicsSystems::Last)
                .in_set(CombatSystemSet::PostPhysicsSet)
                .run_if(in_state(InGameState::InGame)),
        )
        .add_observer(add_xp_collider_components)
        .add_observer(add_health_pickup_collider_components)
        .add_observer(health_pickup)
        .add_observer(award_xp);
    }
}

fn xp_orb_update(
    game_time: Res<Time<Virtual>>,
    mut q_position: Query<(&Position, &mut LinearVelocity, &mut XPPickup)>,
    q_player: Query<&Position, Without<XPPickup>>,
) {
    for (xp_pos, mut xp_lv, mut pickup) in &mut q_position {
        if let Some(t_ent) = pickup.targeting {
            pickup.t_time += game_time.delta_secs();
            if let Ok(t_pos) = q_player.get(t_ent) {
                let dist = (t_pos.0).distance(xp_pos.0);
                let dir = (t_pos.0 - xp_pos.0).normalize_or_zero();
                // shoutout parabolas
                let velo_min =
                    -1.0 * XP_PICKUP_BASE_MOVE_SPEED * (XP_PICKUP_CURVE_TIME_TO_ZERO).powf(2.0);
                let speed = (XP_PICKUP_BASE_MOVE_SPEED) * pickup.t_time.powf(2.0) + velo_min;

                if dist < 10.0 {
                    xp_lv.0 = dist * dir
                } else {
                    xp_lv.0 = speed * dir;
                }
            }
        } else {
            pickup.t_time = 0.0
        }
    }
}

fn add_xp_collider_components(trig: On<Add, XPPickup>, mut commands: Commands) {
    commands.entity(trig.entity).insert((
        CommonColliderBundle::new(
            RigidBody::Kinematic,
            Collider::circle(10.0),
            1.0,
            [ColliderTypes::RemotePickup].into(),
            [ColliderTypes::Player, ColliderTypes::PlayerPickupRadius].into(),
        ),
        Sensor,
        AppliesCollisionEffect::new(
            [ColliderTypes::PlayerPickupRadius].into(),
            XPPickupFollowPlayer,
        ),
        AppliesCollisionEffect::new([ColliderTypes::Player].into(), TriggerPickup),
    ));
}

fn add_health_pickup_collider_components(trig: On<Add, HealthPickup>, mut commands: Commands) {
    commands.entity(trig.entity).insert((
        CommonColliderBundle::new(
            RigidBody::Kinematic,
            Collider::circle(20.0),
            1.0,
            [ColliderTypes::StaticPickup].into(),
            [ColliderTypes::Player].into(),
        ),
        Sensor,
        AppliesCollisionEffect::new([ColliderTypes::Player].into(), TriggerPickup),
    ));
}

#[derive(EntityEvent)]
pub struct PickupTrigger {
    entity: Entity,
    /// Whatever the entity is that ran into the pickup
    pub apply_to: Entity,
}

impl PickupTrigger {
    pub fn new(from: Entity, to: Entity) -> Self {
        Self {
            entity: from,
            apply_to: to,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Reflect)]
pub struct TriggerPickup;
impl CollisionEffect for TriggerPickup {
    fn apply_to(&self, coms: &mut Commands, to: Entity, from: Entity) {
        coms.trigger(PickupTrigger::new(from, to));
    }
}

fn award_xp(
    on: On<PickupTrigger>,
    mut commands: Commands,
    q_trigger: Query<&XPPickup>,
    mut q_lm: Single<&mut XPManager>,
    q_xp: Query<&XPGain>,
) {
    let mult = q_xp.iter().fold(1.0, |acc, xp| acc * xp.0);
    let xp_to_add = if let Ok(pickup) = q_trigger.get(on.entity) {
        commands.entity(on.entity).despawn();
        pickup.val * mult
    } else {
        0.0
    };
    q_lm.c_xp += xp_to_add
}

fn health_pickup(
    on: On<PickupTrigger>,
    mut commands: Commands,
    q_pickup: Query<&HealthPickup>,
    mut q_target: Query<(&mut Health)>,
) {
    if let Ok(pickup) = q_pickup.get(on.entity) {
        if let Ok(mut hp) = q_target.get_mut(on.apply_to) {
            info!("Appyling {:?}", pickup.amount);
            hp.current += pickup.amount;
        }
        commands.entity(on.entity).despawn();
    }
}

fn hp_spawner_update(
    game_time: Res<Time<Virtual>>,
    mut commands: Commands,
    mut q_spawner: Query<(&Position, &mut HealthPickupSpawner)>,
    q_pickups: Query<(), With<HealthPickup>>,
) {
    for (pos, mut spawner) in &mut q_spawner {
        if let Ok(()) = q_pickups.get(spawner.pickup) {
        } else {
            spawner.timer.tick(game_time.delta());
            if spawner.timer.just_finished() {
                let pickup = commands
                    .spawn((
                        HealthPickup {
                            amount: spawner.hp_amount,
                        },
                        *pos,
                    ))
                    .id();
                spawner.pickup = pickup;
                spawner.timer.reset();
            }
        }
    }
}

pub fn tmp_spawn_health_spawner(mut commands: Commands) {
    let pos = Position(Vec2::new(500.0, 400.0));
    let amount = 5.0;
    let pickup = commands.spawn((HealthPickup { amount }, pos)).id();
    let _spawner = commands.spawn((
        HealthPickupSpawner {
            pickup,
            hp_amount: amount,
            timer: Timer::from_seconds(HEALTH_PICKUP_SPAWNER_COOLDOWN, TimerMode::Once),
        },
        pos,
    ));
}
