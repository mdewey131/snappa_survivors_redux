use crate::shared::stats::components::*;
pub trait DisplayableStat {
    fn display_value(&self) -> f32;
}

impl DisplayableStat for AttackRange {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for Armor {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for CritChance {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for CritDamage {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for CooldownRate {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for Damage {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for EffectDuration {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for EffectSize {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for Evasion {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for HealthRegen {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for Luck {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for LifeSteal {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for MovementSpeed {
    fn display_value(&self) -> f32 {
        self.current
    }
}

impl DisplayableStat for PickupRadius {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for ProjectileCount {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for ProjectileSpeed {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for Shield {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for Thorns {
    fn display_value(&self) -> f32 {
        self.0
    }
}

impl DisplayableStat for XPGain {
    fn display_value(&self) -> f32 {
        self.0
    }
}
