use crate::shared::weapons::WeaponKind;
use bevy::prelude::*;

/// The component that marks a given upgrade.
/// Players will be offered one of three choices for them
/// to take, which will boost their stats depending on
/// the upgrade kind (which provides base values),
/// and the rarity (which modifies those values)
#[derive(Component)]
pub struct Upgrade {
    pub kind: UpgradeKind,
    pub rarity: UpgradeRarity,
    pub level: u8,
}

pub enum UpgradeRarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

pub enum UpgradeKind {
    UpgradeWeapon(WeaponKind),
    UpgradeStat(StatUpgradeKind),
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
