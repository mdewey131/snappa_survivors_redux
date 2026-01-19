use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect)]
#[reflect(Default)]
pub struct AttackRange(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect)]
#[reflect(Default)]
pub struct CritChance(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect)]
#[reflect(Default)]
pub struct CritDamage(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect)]
#[reflect(Default)]
pub struct CooldownRate(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect)]
#[reflect(Default)]
pub struct Damage(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect)]
#[reflect(Default)]
pub struct EffectDuration(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect)]
#[reflect(Default)]
pub struct EffectSize(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect)]
#[reflect(Default)]
pub struct Health {
    pub max: f32,
    pub current: f32,
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect)]
#[reflect(Default)]
pub struct Luck(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect)]
#[reflect(Default)]
pub struct MovementSpeed {
    pub current: f32,
    pub cap: f32,
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect)]
#[reflect(Default)]
pub struct PickupRadius(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect)]
#[reflect(Default)]
pub struct ProjectileCount(pub u8);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect)]
#[reflect(Default)]
pub struct ProjectileSpeed(pub f32);

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect)]
#[reflect(Default)]
pub struct XPGain(pub f32);
