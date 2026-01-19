use avian2d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};
use lightyear::{prediction::SyncComponent, prelude::*};
use serde::{Deserialize, Serialize};

use crate::shared::{
    combat::CombatSystemSet,
    damage::{DamageBuffer, DamageInstance},
    states::InGameState,
    stats::components::Damage,
};

pub struct SharedColliderPlugin;

impl Plugin for SharedColliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (tick_rec_collided)
                .in_set(CombatSystemSet::Combat)
                .run_if(in_state(InGameState::InGame)),
        )
        .add_systems(
            FixedPostUpdate,
            (collision_damage_system.after(PhysicsSystems::Last))
                .in_set(CombatSystemSet::PostPhysicsSet)
                .run_if(in_state(InGameState::InGame)),
        );
    }
}

/// Handles the marker components for things like "AppliesCollisionEffect"
pub struct CollidersProtocolPlugin;
impl Plugin for CollidersProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<AppliesCollisionEffect<ApplyDamage>>()
            .add_prediction();
    }
}

/// The common set of things that every entity with a RigidBody in this game must have
#[derive(Bundle)]
pub struct CommonColliderBundle {
    r: RigidBody,
    c: Collider,
    l: LockedAxes,
    m: Mass,
    layers: CollisionLayers,
    en: CollisionEventsEnabled,
}

impl CommonColliderBundle {
    pub fn new(
        r: RigidBody,
        c: Collider,
        mass: f32,
        layer_membership: LayerMask,
        layer_filter: LayerMask,
    ) -> Self {
        Self {
            r,
            c,
            l: LockedAxes::ROTATION_LOCKED,
            m: Mass(mass),
            layers: CollisionLayers::new(layer_membership, layer_filter),
            en: CollisionEventsEnabled,
        }
    }
}

/// Each collider that an entity takes will be a child
/// each of those children will have specified collider sizes
/// and individual filters
///
/// Becuase this has to be a "flat" enum, and because
/// the single player mode exists, this has to make a
/// distinction between predicted and replicated
/// types in each level. I hate it, but I don't know
/// what else to do about it
#[derive(PhysicsLayer, Default, Clone, Copy, Debug)]
pub enum ColliderTypes {
    #[default]
    Player,
    Enemy,
    PlayerProjectile,
    EnemyProjectile,
    PlayerPickupRadius,
    /// Has to be run over by the player
    StaticPickup,
    //Can be picked up by pickup radius
    RemotePickup,
    PlayerRevive,
}

/// There are many things that we may want to have happen upon collision betweeen two units:
///     1. Apply Damage
///     2. Apply Status Effects
///         2a. Slows
///         2b. Stuns
///     3. Trigger a pickup effect (heals, xp gain, etc)
///     4. Trigger some other behavior (start following the thing you collided with)
/// All of these effects have some common origin, where you check the collision between the
/// unit that applies the effect, and the unit to which it applies that effect, and then
/// you want to have that behavior happen
///
/// So, the goal here is to define a common trait that dicates how combat effects get processed,
/// and to have a component that uses that trait object along with the target collider types in order
/// to apply it
///
/// We require SyncComponent:
/// Compnent is required as a shorthand for Send + Sync + 'static
/// the SyncComponent portion guarantees that we can replicate these collision effects
/// between server and client
pub trait CollisionEffect: SyncComponent {
    fn apply_to(&self, coms: &mut Commands, to: Entity, from: Entity);
}

#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug, Reflect)]
#[reflect(from_reflect = false)]
pub struct AppliesCollisionEffect<E> {
    pub to: LayerMask,
    #[reflect(ignore)]
    pub eff: E,
}

impl<E: CollisionEffect> AppliesCollisionEffect<E> {
    pub fn new(applies_to: LayerMask, eff: E) -> Self {
        Self {
            to: applies_to,
            eff,
        }
    }
}

/// Now, let's see how viable this feels
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Reflect)]
pub struct ApplyDamage;

impl CollisionEffect for ApplyDamage {
    fn apply_to(&self, coms: &mut Commands, to: Entity, from: Entity) {
        coms.queue(move |world: &mut World| {
            // Get damage of the entity applying it
            let ent_dam = world.get::<Damage>(from);
            let dam_val = if let Some(d) = ent_dam {
                d.0
            } else {
                return;
            };
            let mut dam_buff = world.get_mut::<DamageBuffer>(to);
            if let Some(ref mut db) = dam_buff {
                db.push(DamageInstance {
                    damage_source: from,
                    amount: dam_val,
                });
            }
        });
    }
}

/// Applying damage is a more special case of collision based effects than I may have initially appreciated.
/// In this system, we want to add entities to the list in the event that they collide with the entity, but are
/// not in the list of entities that have been recently collided with.
/// This can happen because of collision start, or as the result of an ongoing collision. So, we use the Collisions param
fn collision_damage_system(
    collisions: Collisions,
    mut commands: Commands,
    q_applies_damage: Query<&AppliesCollisionEffect<ApplyDamage>>,
    mut q_damage_target: Query<(Entity, &mut RecentlyCollided, &CollisionLayers)>,
) {
    for (ent_to_damage, mut recent_collided, layers) in &mut q_damage_target {
        for contacts in collisions.collisions_with(ent_to_damage) {
            let applying_entity = if contacts.collider1 == ent_to_damage {
                contacts.collider2
            } else {
                contacts.collider1
            };
            if let Ok(applies_effect) = q_applies_damage.get(applying_entity) {
                if (layers.memberships.0 & applies_effect.to.0) != 0 {
                    if recent_collided.with.get(&applying_entity).is_none() {
                        info!(
                            "Firing collision damage from {:?} to {:?}",
                            applying_entity, ent_to_damage
                        );
                        recent_collided
                            .with
                            .insert(applying_entity, CollisionDamageTimer::new());
                        applies_effect
                            .eff
                            .apply_to(&mut commands, ent_to_damage, applying_entity);
                    }
                }
            }
        }
    }
}

fn tick_rec_collided(
    time: Res<Time<Fixed>>,
    mut q_recents: Query<(Entity, &mut RecentlyCollided)>,
) {
    for (ent, mut recent) in &mut q_recents {
        let mut to_rm = Vec::new();
        for (ent, ref mut timer) in recent.with.iter_mut() {
            timer.tick(time.delta());
            info!(
                "Ticking timer. Remaining: {:?}, game time duration: {:?}",
                timer.remaining_secs(),
                time.delta()
            );
            if timer.just_finished() {
                to_rm.push(ent.clone())
            }
        }
        for ent in to_rm {
            recent.with.remove(&ent);
        }
    }
}

/// A component that stores all of the entities that this entity has recently collided with.
#[derive(Component, Default)]
pub struct RecentlyCollided {
    pub with: HashMap<Entity, CollisionDamageTimer>,
}

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct CollisionDamageTimer(pub Timer);
impl CollisionDamageTimer {
    pub fn new() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Once))
    }
}
