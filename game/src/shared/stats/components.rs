use crate::shared::damage::DamageBuffer;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct AttackRange(pub f32);

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
#[require(DamageBuffer)]
pub struct Health {
    pub max: f32,
    pub current: f32,
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct Luck(pub f32);

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
pub struct XPGain(pub f32);

pub struct StatsProtocolPlugin;

impl Plugin for StatsProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<AttackRange>().add_prediction();
        app.register_component::<CritChance>().add_prediction();
        app.register_component::<CritDamage>().add_prediction();
        app.register_component::<CooldownRate>().add_prediction();
        app.register_component::<Damage>().add_prediction();
        app.register_component::<EffectSize>().add_prediction();
        app.register_component::<EffectDuration>().add_prediction();
        app.register_component::<Health>().add_prediction();
        app.register_component::<Luck>().add_prediction();
        app.register_component::<MovementSpeed>().add_prediction();
        app.register_component::<PickupRadius>().add_prediction();
        app.register_component::<ProjectileCount>().add_prediction();
        app.register_component::<ProjectileSpeed>().add_prediction();
        app.register_component::<XPGain>().add_prediction();
    }
}
