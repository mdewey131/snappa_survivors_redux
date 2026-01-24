//! A proof of concept around generating relationships in stats between entities
//!
//! In order for this to work, entities need to maintain two different versions of a stat:
//! 1. The base value
//! 2. The current value
//!
//! We trigger recalculations on a stat whenever one of the following is true:
//! 1. The base value of the stat has changed for this entity
//! 2. The current value of a stat that affects "this one" has changed
//!
//! This means that we need to keep record of which current stats affect others, and
//! for efficiency of lookup reasons, this should probably be two sided (we need to know to "trigger recalcs")
//! on dependent stats, and dependent stats need to be able to efficiently look up the components
//! that modify them
use std::marker::PhantomData;

use super::{StatKind, components::*};

use super::components::Base;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

/// All of the stats that modify "this one"
#[derive(Component, Debug, Clone)]
pub struct StatModifiedBy<C> {
    pub list: Vec<StatModifierDependency>,
    _mark: PhantomData<C>,
}
/// This uses the stat kind enum for the sake of knowing what to grab,
/// but it doesn't actually use the value
#[derive(Component, Debug, Clone, Copy)]
pub struct StatModifierDependency {
    pub ent: Entity,
    pub stat: StatKind,
    pub cat: StatCategory,
    pub method: StatModifierMethod,
}

/// All of the stats that "this one" modifies
#[derive(Component, Debug, Clone)]
pub struct ModifiesStats<C> {
    pub list: Vec<StatModifierTarget>,
    _mark: PhantomData<C>,
}

#[derive(Component, Debug, Clone)]
pub struct StatModifierTarget {
    pub ent: Entity,
    pub stat: StatKind,
    pub method: StatModifierMethod,
}

pub fn recalculate_stat<S: Stat>(
    mut param_set: ParamSet<(
        Query<(Entity, &mut Current<S>, &Base<S>, &StatModifiedBy<S>)>,
        Query<EntityRef>,
    )>,
) {
    // Calculate a list of operations that we need to perform per entity by pulling out of the modifiers list
    // Doing it this way should allow us to access entity mut later on?
    let mut to_dos = HashMap::new();
    let mut modifiers = HashMap::new();

    for (ent_to_mod, _c_stat, base_stat, stat_mod_records) in param_set.p0().iter() {
        to_dos.insert(
            ent_to_mod,
            (base_stat.0.get(), stat_mod_records.list.clone()),
        );
    }

    for (ent_to_mod, (base_stat, modifier_list)) in to_dos.iter() {
        // Initialize with 0
        modifiers.insert(ent_to_mod, 0.0);
        for modifier in modifier_list.iter() {
            let q_eref = param_set.p1();
            let ent_ref = q_eref.get(modifier.ent);
            if let Ok(e_ref) = ent_ref {
                let val = get_component_value(e_ref, modifier.stat, modifier.cat);
                if let Some(v) = val {
                    let to_apply = match modifier.method {
                        StatModifierMethod::FlatAddStat => v,
                        StatModifierMethod::StatMultipliesBase => v * base_stat,
                    };
                    modifiers
                        .entry(ent_to_mod)
                        .and_modify(|modif| *modif += to_apply);
                }
            }
        }
    }

    // Now, bring back the stats and modify them
    for (ent_to_mod, mut c_stat, base_stat, _stat_mod_records) in param_set.p0().iter_mut() {
        let total_mod_value = modifiers.get(&ent_to_mod).expect("Not found?");
        c_stat.0.set(base_stat.0.get() + total_mod_value)
    }
}

fn get_component_value(e_ref: EntityRef, stat_kind: StatKind, cat: StatCategory) -> Option<f32> {
    match stat_kind {
        StatKind::Armor(_) => get_component_value_from_cat::<Armor>(e_ref, cat),
        StatKind::AttackRange(_) => get_component_value_from_cat::<AttackRange>(e_ref, cat),
        StatKind::CDR(_) => get_component_value_from_cat::<CooldownRate>(e_ref, cat),
        StatKind::CritChance(_) => get_component_value_from_cat::<CritChance>(e_ref, cat),
        StatKind::CritDamage(_) => get_component_value_from_cat::<CritDamage>(e_ref, cat),
        StatKind::Damage(_) => get_component_value_from_cat::<Damage>(e_ref, cat),
        StatKind::EffDuration(_) => get_component_value_from_cat::<EffectDuration>(e_ref, cat),
        StatKind::EffSize(_) => get_component_value_from_cat::<EffectSize>(e_ref, cat),
        StatKind::Evasion(_) => get_component_value_from_cat::<Evasion>(e_ref, cat),
        StatKind::Health(_) => get_component_value_from_cat::<Health>(e_ref, cat),
        StatKind::HealthRegen(_) => get_component_value_from_cat::<HealthRegen>(e_ref, cat),
        StatKind::LifeSteal(_) => get_component_value_from_cat::<LifeSteal>(e_ref, cat),
        StatKind::Luck(_) => get_component_value_from_cat::<Luck>(e_ref, cat),
        StatKind::MS(_) => get_component_value_from_cat::<MovementSpeed>(e_ref, cat),
        StatKind::PickupR(_) => get_component_value_from_cat::<PickupRadius>(e_ref, cat),
        StatKind::ProjCount(_) => get_component_value_from_cat::<ProjectileCount>(e_ref, cat),
        StatKind::ProjSpeed(_) => get_component_value_from_cat::<ProjectileSpeed>(e_ref, cat),
        StatKind::Shield(_) => get_component_value_from_cat::<Shield>(e_ref, cat),
        StatKind::Thorns(_) => get_component_value_from_cat::<Thorns>(e_ref, cat),
        StatKind::XPGain(_) => get_component_value_from_cat::<XPGain>(e_ref, cat),
    }
}

fn get_component_value_from_cat<S: Stat>(e_ref: EntityRef, cat: StatCategory) -> Option<f32> {
    match cat {
        StatCategory::Base => e_ref.get_components::<&Base<S>>().map(|comp| comp.0.get()),
        StatCategory::Current => e_ref
            .get_components::<&Current<S>>()
            .map(|comp| comp.0.get()),
    }
}
