//! The perennial overhaul of the weapons system.
//!
//! At present, players have the ability to use weapons that are dependent on a stats relationship.
//! Additionally, they have the ability to unlock passive skills that just boost singular stats.
//! These systems not only feel clunky, but they're too much of a special case
//!
//! 1. What if I want players to have abilities that are controlled by a button press?
//! 2. What about enemies? Shouldn't they be able to do things other than walk at a player?
//!
//! This script explores the viability of unifying together a lot of these concepts into "Abilities".
//!
//!
//! Things that abilites should be able to do:
//!
//! 1. Passively boost the stats of an entity
//! 2. Be activateable by input as well as by the game automatically when appropriate
//! 3. Create a group of projectiles when activated, and possibly despawn them when deactivated
//! 4. Start a timer to activate every completion, with an arbitrary number of ticks including infinite
//! 5.
use std::marker::PhantomData;

use bevy::{
    ecs::{event::Trigger, query::QueryFilter},
    prelude::*,
};
use serde::Deserialize;

use crate::shared::{
    combat::Cooldown,
    stats::components::{CooldownRate, ProjectileCount},
};

/// A component that holds the output of each individual logical check component on an ability.
/// These need to be writen to and drained every update
pub struct AbilityTriggerConditions {
    to_start: Vec<bool>,
}


/// Simple marker
#[derive(Component, Clone, Debug, Default)]
struct Active;

fn set_ability_active(
    mut commands: Commands,
    q_ability: Query<(Entity, &AbilityTriggerConditions), Without<Active>>,
) {
    for (ability, conditions) in &q_ability {
        if conditions.to_start.iter().all(|start| start) {
            commands.entity(ability).insert(Active);
        }
    }
}


/// Returns true if the entity is in the queryfilter provided, else false
pub struct CheckSelfFor<QF: QueryFilter> {
    _mark: PhantomData<QF>
}

pub fn check_self_for<QF: QueryFilter>(
    q_self: Query<(Entity, &mut AbilityTriggerConditions With<
)



/// This always checks its own range, which may be a problem in the
/// future if we want to check this from a player or enemy's pov
#[derive(Component, Clone, Debug, Default)]
pub struct CheckIfEntitiesInRange<QF: QueryFilter> {
    in_range: bool,
    _mark: PhantomData<QF>
}


/// Any ability with this component will tick down while it doesn't have a CD
///
/// This component expects to be the only active condition, becuase it will st
#[derive(Component, Clone, Debug, Default, Deref, DerefMut)]
pub struct ActiveForTimer(Option<Timer>);


/// Any ability with this component will activate on a pulse
#[derive(Component, Clone, Debug, Default, )]
pub struct PulseActivate {
    rem_pulses: Option<u8>,
    tickrate: Timer,
}

/// Other things are responsible for taking this away
#[derive(Component, Clone, Debug, Default)]
pub struct HoldsCharges {
    max_charges: u8,
    c_charges: u8,
    tickrate: Timer,
    deactivate_on_zero_charges: bool,
}




pub fn ability_active_for_timer(
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    mut q_timer: Query<(&mut ActiveForTimer), With<Active>>
) {
    for mut t in q_timer {

    }
}




#[derive(Component, Debug, Clone)]
#[require(AbilityKind = Ability)]
pub struct DiceGuard {
    active_timer: Timer,
    projectiles: Option<Vec<Entity>>,
}

#[derive(Component, Debug, Clone)]
pub enum AbilityKind {
    PassiveHearty,
    DiceGuard ,
    ShiftyShot,
    PaddleBack,
}

fn tick_dice_guard(
    q_guard: Query<(&mut DiceGuard, &CooldownRate, &ProjectileSpeed, &Damage, Option<&Cooldown>, )
) -> {

}








pub fn update_abilties(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut q_abilities: Query<(
        &mut Ability,
        Option<&Cooldown>,
        Option<&CooldownRate>,
        Option<&ProjectileCount>,
    )>,
) {
    for (mut ability, m_cd, m_cdr, m_proj) in &mut q_abilities {
        let first_trigger = !ability.is_active && m_cd.is_none();
        // We can run the update
        if ability.is_active || first_trigger {}
    }
}

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = AbilityOf)]
pub struct HasAbilities(Vec<Entity>);

#[derive(Component, Debug, Clone)]
#[relationship(relationship_target = HasAbilities)]
pub struct AbilityOf(Entity);
