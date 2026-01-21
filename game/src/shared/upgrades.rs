use crate::shared::weapons::WeaponKind;
use bevy::prelude::*;

#[derive(Component)]
pub struct Upgrade {
    pub kind: UpgradeKind,
}

pub enum UpgradeKind {
    UpgradeWeapon(WeaponKind),
    UpgradeStat(StatUpgrade),
}
pub struct StatUpgrade {
    pub sk: StatUpgradeKind,
    pub v: f32,
}

pub enum StatUpgradeKind {
    Armor,
    CritChance,
    CDR,
    Damage,
    EffDuration,
    EffSize,
    Evasion,
    MaxHealth,
    HealthRegen,
    Luck,
    MoveSpeed,
    PickupRadius,
    ProjectileCount,
    Shield,
    Thorns,
    XPGain,
}
