use crate::shared::{damage::* /*stats::components::Damage*/};
use avian2d::prelude::*;
use bevy::prelude::*;
use lightyear::prediction::SyncComponent;
use serde::{Deserialize, Serialize};
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
