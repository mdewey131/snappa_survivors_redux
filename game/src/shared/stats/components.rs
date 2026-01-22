use crate::shared::damage::DamageBuffer;
use bevy::{ecs::component::Mutable, prelude::*};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// Uses trait defined getters and setters because we can't guarantee what the
/// internals look like, and this allows us to recalculate the stats via relationships
/// in a generic way
pub trait Stat: Component {
    fn get(&self) -> f32;
    fn set(&mut self, val: f32);
}

/// A wrapper component around the base version of a stat
#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
pub struct Base<S: Stat>(pub S);

/// A wrapper component around the current value of a stat
#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
pub struct Current<S: Stat>(pub S);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct AttackRange(pub f32);
impl Stat for AttackRange {
    fn get(&self) -> f32 {
        self.0
    }
    fn set(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct Armor(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct CritChance(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct CritDamage(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct CooldownRate(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct Damage(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct EffectDuration(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct EffectSize(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct Evasion(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
#[require(DamageBuffer)]
pub struct Health {
    pub max: f32,
    pub current: f32,
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct HealthRegen(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct Luck(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct LifeSteal(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct MovementSpeed {
    pub current: f32,
    pub cap: f32,
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct PickupRadius(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct ProjectileCount(pub u8);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct ProjectileSpeed(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct Shield(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct Thorns(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct XPGain(pub f32);
