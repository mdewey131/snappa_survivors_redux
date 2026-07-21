use crate::shared::{
    damage::{DamageBuffer, DamageInstance},
    stats::{StatKind, StatList},
};
use bevy::{
    ecs::component::{ComponentMutability, Mutable},
    prelude::*,
};
use serde::{Deserialize, Serialize};

/// Marks a compoonent as being dependent on a stat.
///
/// It is the responsibility of this component to check up on the value of statlist
/// every frame. Doing it this way means that we're not required to carry a mutable reference
/// to the whole stats list anytime we want to read a particular stat.
/// TODO: Rework the internals of this, its weird
pub trait StatComponent: Component<Mutability = Mutable> + Sized + Clone + Copy {
    /// TODO: Implement this as Into<StatKind> on the trait boundary, make a proc(?) macro for it
    fn stat_kind(&self) -> StatKind;

    /// Takes a mutable reference to self and makes selective updates where necessary
    fn update_self(&mut self, val: f32);

    fn update_stat_component(mut q_self: Query<(&mut Self, &mut StatList)>) {
        for (mut stat_comp, mut list) in &mut q_self {
            let self_sk: StatKind = stat_comp.stat_kind();
            if let Some(stat) = list.get_current(&self_sk) {
                stat_comp.update_self(stat);
            }
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
#[require(DamageBuffer)]
pub struct Health {
    max: f32,
    pub current: f32,
}
impl Health {
    pub fn new(val: f32) -> Self {
        Self {
            max: val,
            current: val,
        }
    }
    pub fn max(&self) -> f32 {
        self.max
    }
}

impl StatComponent for Health {
    fn stat_kind(&self) -> StatKind {
        StatKind::Health
    }
    fn update_self(&mut self, val: f32) {
        if self.max == val {
        } else {
            let c_pct = self.current / self.max;
            self.current = val * c_pct;
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct AttackRange(pub f32);

impl StatComponent for AttackRange {
    fn stat_kind(&self) -> StatKind {
        StatKind::AttackRange
    }
    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct Armor(pub f32);

impl StatComponent for Armor {
    fn stat_kind(&self) -> StatKind {
        StatKind::Armor
    }
    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}
impl Armor {
    pub fn mitigate_incoming_damage(&self, incoming: f32) -> f32 {
        // Apply the armor equation, whatever that is.
        // Want armor to be diminishing returns in damage mitigation.
        // going to blatantly take an equation from LoL just for the purposes of understanding at the moment
        return incoming * (100.0 / (100.0 + self.0));
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct CritChance(pub f32);

impl StatComponent for CritChance {
    fn stat_kind(&self) -> StatKind {
        StatKind::CritChance
    }
    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct CritDamage(pub f32);

impl StatComponent for CritDamage {
    fn stat_kind(&self) -> StatKind {
        StatKind::CritDamage
    }
    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct CooldownRate(pub f32);

impl StatComponent for CooldownRate {
    fn stat_kind(&self) -> StatKind {
        StatKind::CDR
    }
    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct Damage(pub f32);

impl StatComponent for Damage {
    fn stat_kind(&self) -> StatKind {
        StatKind::Damage
    }
    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct EffectDuration(pub f32);

impl StatComponent for EffectDuration {
    fn stat_kind(&self) -> StatKind {
        StatKind::EffDuration
    }
    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct EffectSize(pub f32);
impl StatComponent for EffectSize {
    fn stat_kind(&self) -> StatKind {
        StatKind::EffSize
    }
    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct Evasion(pub f32);

impl StatComponent for Evasion {
    fn stat_kind(&self) -> StatKind {
        StatKind::Evasion
    }
    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct HealthRegen(pub f32);
impl StatComponent for HealthRegen {
    fn stat_kind(&self) -> StatKind {
        StatKind::HealthRegen
    }
    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct Luck(pub f32);
impl StatComponent for Luck {
    fn stat_kind(&self) -> StatKind {
        StatKind::Luck
    }
    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct LifeSteal(pub f32);

impl StatComponent for LifeSteal {
    fn stat_kind(&self) -> StatKind {
        StatKind::LifeSteal
    }

    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct MovementSpeed {
    pub current: f32,
    pub cap: f32,
}

impl StatComponent for MovementSpeed {
    fn stat_kind(&self) -> StatKind {
        StatKind::MS
    }

    fn update_self(&mut self, val: f32) {
        self.current = val.clamp(0.0, self.cap)
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct PickupRadius(pub f32);

impl StatComponent for PickupRadius {
    fn stat_kind(&self) -> StatKind {
        StatKind::PickupR
    }

    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}
#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct ProjectileBounces(pub f32);

impl StatComponent for ProjectileBounces {
    fn stat_kind(&self) -> StatKind {
        StatKind::ProjBounces
    }

    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct Revive(pub f32);

impl StatComponent for Revive {
    fn stat_kind(&self) -> StatKind {
        StatKind::Revive
    }

    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct ProjectileCount(pub f32);
impl StatComponent for ProjectileCount {
    fn stat_kind(&self) -> StatKind {
        StatKind::ProjCount
    }

    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct ProjectileSpeed(pub f32);
impl StatComponent for ProjectileSpeed {
    fn stat_kind(&self) -> StatKind {
        StatKind::ProjSpeed
    }

    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct Shield(pub f32);

impl StatComponent for Shield {
    fn stat_kind(&self) -> StatKind {
        StatKind::Shield
    }
    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct Thorns(pub f32);

impl StatComponent for Thorns {
    fn stat_kind(&self) -> StatKind {
        StatKind::Thorns
    }

    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default, Reflect, PartialEq)]
#[reflect(Default)]
pub struct XPGain(pub f32);

impl StatComponent for XPGain {
    fn stat_kind(&self) -> StatKind {
        StatKind::XPGain
    }

    fn update_self(&mut self, val: f32) {
        self.0 = val
    }
}
