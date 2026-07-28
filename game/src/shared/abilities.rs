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
use std::{marker::PhantomData, time::Duration};

use avian2d::{dynamics::rigid_body::LinearVelocity, physics_transform::Position};
use bevy::{
    ecs::{event::Trigger, query::QueryFilter},
    prelude::*,
    transform::commands,
};
use serde::{Deserialize, Serialize};

use crate::{
    client::main_menu::MainMenuScreen,
    shared::{
        colliders::{AppliesCollisionEffect, ApplyDamage, ColliderTypes},
        combat::{CombatSystemSet, Cooldown},
        damage::HealthBuffer,
        enemies::Enemy,
        game_kinds::{self, CurrentGameKind, GameKinds::SinglePlayer, MultiPlayerComponentOptions},
        game_object_spawning::{SpawnGameObject, spawn_game_object},
        players::Player,
        projectiles::{Projectile, ProjectileMovement},
        states::{AppState, InGameState},
        stats::components::{
            AttackRange, CooldownRate, CritChance, CritDamage, Damage, EffectDuration, EffectSize,
            Health, ProjectileBounces, ProjectileCount, ProjectileSpeed,
        },
        weapons::{DiceGuardProjectile, ShiftyShotAttack, find_closest_in_list},
    },
    utils::CreatedBy,
};

#[derive(Component, Debug, Clone, Default)]
pub struct NewAbility {
    /// When this doesn't have steps, it tracks its own state as it goes along
    /// When it does have steps, it tracks them in sequence based on `HasAbilitySteps`
    pub state: NewAbilityState,
}
#[derive(Component, Debug, Clone, Default)]
#[relationship_target(relationship = AbilityStep)]
pub struct HasAbilitySteps {
    pub current: usize,
    pub state: NewAbilityState,
    pub cancels_entire_ability_on_self_cancel: bool,
    #[relationship]
    steps: Vec<Entity>,
}

#[derive(Component, Debug, Clone, Copy)]
#[relationship(relationship_target = HasAbilitySteps)]
pub struct AbilityStep {
    #[relationship]
    pub entity: Entity,
    pub state: NewAbilityState,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub enum NewAbilityState {
    #[default]
    Init,
    Requested,
    /// Held here for one frame in case a system needs to have an effect on start
    Started,
    /// This step is executing. Will want to think about how starting works
    Executing,
    /// Imagine a channeled ability gets cancelled, or a tether breaks, etc
    Cancelled,
    Completed,
    /// This wasn't allowed to execute
    Failure,
}

#[derive(Message, Component, Debug, Clone, Default)]
pub enum AbilityFailureReason {
    #[default]
    Default,
    FailedValidation {
        validators: Vec<Entity>,
    },
    Cancelled,
}

#[derive(Component)]
#[relationship(relationship_target = StartedBy)]
pub struct ConditionStarts(Entity);

/// Placed on an ability step to validate that the step is allowed to continue
#[derive(Component)]
#[relationship_target(relationship = ConditionStarts)]
pub struct StartedBy(Vec<Entity>);

#[derive(Component, Debug)]
#[relationship(relationship_target = CancelledBy)]
pub struct ConditionCancels(pub Entity);
/// Holds a list of ability validators that, if triggered, cause this ability to be cancelled
/// This implies `ConditionKind::AnyTrue`
#[derive(Component, Debug)]
#[relationship_target(relationship = ConditionCancels)]
pub struct CancelledBy(Vec<Entity>);

#[derive(Component, Debug, Default)]
#[relationship_target(relationship = ConditionCompletes)]
pub struct CompletedBy {
    pub join: JoinCondition,
    #[relationship]
    ents: Vec<Entity>,
}
#[derive(Component, Debug)]
#[relationship(relationship_target = CompletedBy)]
pub struct ConditionCompletes(pub Entity);

#[derive(Component, Debug, Default)]
pub enum JoinCondition {
    #[default]
    Any,
    All,
}

fn single_step_ability(
    mut q_ability: Query<
        (
            &mut NewAbility,
            Option<&StartedBy>,
            Option<&CancelledBy>,
            Option<&CompletedBy>,
            Option<&AutoCast>,
        ),
        Without<HasAbilitySteps>,
    >,
    q_validators: Query<&AbilityValidator>,
) {
    for (mut ability, m_validators, m_cancels, m_completions, m_auto) in &mut q_ability {
        let all_validations_true = if let Some(v) = m_validators {
            v.iter()
                .all(|ent| q_validators.get(ent).expect("Wat").value)
        } else {
            true
        };
        let any_cancels_true = if let Some(c) = m_cancels {
            c.iter()
                .any(|ent| q_validators.get(ent).expect("not found").value)
        } else {
            false
        };
        match ability.state {
            NewAbilityState::Init => {
                if m_auto.is_some() && all_validations_true {
                    ability.state = NewAbilityState::Executing
                } else if m_auto.is_some() {
                    ability.state = NewAbilityState::Requested;
                }
            }
            NewAbilityState::Requested => {
                if all_validations_true {
                    ability.state = NewAbilityState::Started
                } else {
                    ability.state = NewAbilityState::Failure
                }
            }
            NewAbilityState::Started => {
                if any_cancels_true {
                    ability.state = NewAbilityState::Cancelled
                } else {
                    ability.state = NewAbilityState::Executing
                }
            }
            NewAbilityState::Executing => {
                let met_completion_conditions = if let Some(comps) = m_completions {
                    match comps.join {
                        JoinCondition::Any => comps
                            .ents
                            .iter()
                            .any(|ent| q_validators.get(*ent).expect("?").value),
                        JoinCondition::All => comps
                            .ents
                            .iter()
                            .all(|ent| q_validators.get(*ent).expect("?").value),
                    }
                } else {
                    false
                };

                if any_cancels_true {
                    ability.state = NewAbilityState::Cancelled
                } else if met_completion_conditions {
                    ability.state = NewAbilityState::Completed
                }
            }
            NewAbilityState::Completed => ability.state = NewAbilityState::Init,
            NewAbilityState::Cancelled => ability.state = NewAbilityState::Init,
            NewAbilityState::Failure => ability.state = NewAbilityState::Init,
        }
    }
}

fn multi_step_ability(
    mut commands: Commands,
    mut q_ability: Query<(&mut NewAbility, &mut HasAbilitySteps)>,
    q_steps: Query<
        (
            &AbilityStep,
            Option<&AutoCast>,
            Option<&StartedBy>,
            Option<&CancelledBy>,
            Option<&CompletedBy>,
        ),
        Without<NewAbility>,
    >,
    q_validators: Query<&AbilityValidator>,
) {
    for (mut ability, mut steps) in q_ability {
        match ability.state {
            AbilityState::Init => {
                if m_auto.is_some() {
                    Abili
                }
            }
        }
    }
}

/// Recursively walks through the steps and figures out the machinery, making
/// changes to the `HasActionSteps` component along the way.
///
/// This function returns the state that the overall calling ability shuold have
pub fn recursive_ability_step_state_machine(
    commands: &mut Commands,
    steps: &mut HasAbilitySteps,
    q_steps: &Query<
        (
            &AbilityStep,
            Option<&AutoCast>,
            Option<&StartedBy>,
            Option<&CancelledBy>,
            Option<&CompletedBy>,
        ),
        Without<NewAbility>,
    >,
    q_validators: &Query<&AbilityValidator>,
) -> NewAbilityState {
    if steps.current >= steps.steps.len() {
        return NewAbilityState::Completed;
    }
    let c_ent = steps.steps[steps.current];
    let (c_step, m_auto, m_starts, m_cancels, m_completes) =
        q_steps.get(c_ent).expect("Where step");
    let all_starting_conditions_met = if let Some(s) = m_starts {
        s.0.iter().all(|e| q_validators.get(*e).unwrap().value)
    } else {
        true
    };
    let any_cancels = if let Some(c) = m_cancels {
        c.0.iter().any(|e| q_validators.get(*e).unwrap().value)
    } else {
        false
    };
    let completion_conditions_met = if let Some(c) = m_completes {
        match c.join {
            JoinCondition::All => c.ents.iter().all(|e| q_validators.get(*e).unwrap().value),
            JoinCondition::Any => c.ents.iter().any(|e| q_validators.get(*e).unwrap().value),
        }
    } else {
        false
    };
    let step_next_state = match c_step.state {
        NewAbilityState::Init => {
            if m_auto.is_some() && all_starting_conditions_met {
                NewAbilityState::Started
            } else if m_auto.is_some() {
                NewAbilityState::Requested
            } else {
                NewAbilityState::Init
            }
        }
        NewAbilityState::Requested => {
            if all_starting_conditions_met {
                NewAbilityState::Started
            } else {
                NewAbilityState::Failure
            }
        }
        NewAbilityState::Started => {
            if any_cancels {
                NewAbilityState::Cancelled
            } else if completion_conditions_met {
                NewAbilityState::Completed
            } else {
                NewAbilityState::Executing
            }
        }
        NewAbilityState::Executing => {
            if completion_conditions_met {
                NewAbilityState::Completed
            } else if any_cancels {
                NewAbilityState::Cancelled
            } else {
                NewAbilityState::Executing
            }
        }
        NewAbilityState::Cancelled => NewAbilityState::Init,
        NewAbilityState::Failure => NewAbilityState::Init,
    };
}

/*
#[derive(Component, Debug)]
#[relationship(relationship_target = NewAbility)]
pub struct AbilityStep {
    state: AbilityState,
    #[relationship]
    pub entity: Entity
}




/// Hi, I'm an ability
#[derive(Component, Debug, Default)]
#[relationship_target(relationship = AbilityStep)]
pub struct NewAbility {
    pub state: AbilityState,
    pub current_step: usize,
    #[relationship]
    steps: Vec<Entity>,
}

impl NewAbility {
    pub fn request_ability(&self, commands: &mut Commands) {

    }
}
*/
/// I will automatically move to ActionState::Requested if I'm not already in `Requested` or `Executing`
#[derive(Component, Debug, Clone, Copy)]
pub struct AutoCast;

#[derive(Component, Debug, Clone, Copy, Default)]
#[require(AbilityValidator = AbilityValidator::default())]
pub struct AbilityOffCooldown;

fn check_cooldown_validator(
    mut q_validator: Query<(&mut AbilityValidator, &ValidatorOf), With<AbilityOffCooldown>>,
    q_abilities: Query<&mut NewAbility, Without<Cooldown>>,
    q_step: Query<&StepOf>,
) {
    for (mut validator, holder) in &mut q_validator {
        let off_cooldown = if q_abilities.get(holder.entity).is_ok() {
            true
        } else {
            if let Ok(ability) = q_step.get(holder.entity) {
                q_abilities.get(ability.0).is_ok()
            } else {
                false
            }
        };
        validator.value = off_cooldown
    }
}

/// A validator returning true if at least one entity with the filter type is in range
}
pub struct EnemyInAttackRange

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct AddCooldownOnCompletion;
fn add_cooldown_on_ability_completion(
    mut commands: Commands,
    q_ability: Query<(Entity, &NewAbility, &CooldownRate), Without<Cooldown>>,
) {
    for (a_ent, ability, cdr) in &q_ability {
        match ability.state {
            NewAbilityState::Completed => {
                commands.entity(a_ent).insert(Cooldown::new(cdr.0));
            }
            _ => {}
        }
    }
}

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Clone, Copy, Default)]
pub enum AbilitySystemSet {
    #[default]
    CheckValidators,
    /// Handles movement of things from, e.g., "executing" to "completed"
    CheckAbilities,
    /// E.g. things like "add cooldown to entities that have completed"
    StateCheckingSystems,
    /// These systems reset things to their proper place using the state machinery of these abilities
    ResolveAbilityState,
}

pub struct AbilityPlugin;
impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            FixedUpdate,
            (
                AbilitySystemSet::CheckValidators,
                AbilitySystemSet::CheckAbilities,
                AbilitySystemSet::StateCheckingSystems,
                AbilitySystemSet::ResolveAbilityState,
            )
                .chain()
                .in_set(CombatSystemSet::Combat),
        )
        .add_systems(
            FixedUpdate,
            (
                (check_cooldown_validator).in_set(AbilitySystemSet::CheckValidators),
                (active_for_timer, pulse_activation).in_set(AbilitySystemSet::CheckAbilities),
                add_cooldown_on_ability_completion.in_set(AbilitySystemSet::StateCheckingSystems),
                (single_step_ability, multi_step_ability)
                    .in_set(AbilitySystemSet::ResolveAbilityState),
            ),
        );
    }
}

// This will be active for the time in timer, then move to completed
#[derive(Component, Debug, Clone, Default, Reflect)]
pub struct ActiveForTime(Timer);

fn active_for_timer(time: Res<Time>, mut q_ability: Query<(&mut NewAbility, &mut ActiveForTime)>) {
    for (mut ability, mut timer) in &mut q_ability {
        match ability.state {
            NewAbilityState::Executing => {
                timer.0.tick(time.delta());
                if timer.0.just_finished() {
                    ability.state = NewAbilityState::Completed;
                }
            }
            NewAbilityState::Cancelled | NewAbilityState::Completed | NewAbilityState::Failure => {
                timer.0.reset()
            }
            _ => {}
        }
    }
}

// This will be active for the entire duration, and send ActivateAbility messages
// when the timer is up.
#[derive(Component, Debug, Clone, Default, Reflect)]
pub struct PulseActivation {
    pub c_ticks: u8,
    pub timer: Timer,
}

fn pulse_activation(
    mut commands: Commands,
    time: Res<Time>,
    mut q_pulse: Query<(
        Entity,
        &mut NewAbility,
        &mut PulseActivation,
        &ProjectileCount,
    )>,
) {
    for (a_ent, mut ability, mut pulse, p_count) in &mut q_pulse {
        match ability.state {
            NewAbilityState::Init => pulse.c_ticks = p_count.0 as u8,
            NewAbilityState::Executing => {
                pulse.timer.tick(time.delta());
                if pulse.timer.just_finished() {
                    if pulse.c_ticks == 0 {
                        ability.state = NewAbilityState::Completed
                    } else {
                        pulse.c_ticks -= 1;
                    }
                    pulse.timer.reset();
                    commands.trigger(ActivateAbility { entity: a_ent });
                }
            }
            _ => {}
        }
    }
}

fn new_dice_guard_scene() -> impl SceneList {
    bsn_list! [(
        #DummyPlayer1
        Position(Vec2::ZERO)
        HasAbilities [ (
                #DiceGuard
                DiceGuard
                AttackRange(100.0)
                EffectSize(50.0)
                EffectDuration(5.0)
                ProjectileCount(3.0)
                ProjectileSpeed(20.0)
                CooldownRate(3.0)
                Damage(5.0)
                AutoCast
                NewAbility
                ActiveForTime(Timer::from_seconds(5.0, TimerMode::Once))
                TriggerStartAbility
                TriggerEndAbility
                HasValidators [(
                    #OffCDValidator
                    AbilityOffCooldown
                )
                ]

                on(generic_observer)
                on(dice_guard_activate)
                on(dice_guard_deactivate)
        )]
        ),
        (
            #DiceGuardEnemy
            Enemy
            Position(Vec2::new(0.0, 100.0))
        )
    ]
}

pub fn new_shifty_shot_scene(position: Vec2) -> impl SceneList {
    let e1_pos = position + Vec2::Y * 250.0;
    let e2_pos = position + Vec2::Y * 250.0 + Vec2::X * 100.0;
    bsn_list! [(
        #DummyPlayer
        Player
        Position(position)
        Transform {translation: Vec3::new(400.0, 0.0, 0.0)}
        HasAbilities [
            #ShiftyShot
            AttackRange(250.0)
            ProjectileBounces(1.0)
            ProjectileCount(3.0)
            ProjectileSpeed(50.0)
            CooldownRate(3.0)
            CritChance(0.5)
            CritDamage(0.5)
            Damage(5.0)
            NewAbility
            AutoCast
            TriggerStartAbility
            PulseActivation {
                c_ticks: 3,
                timer: Timer::from_seconds(1.0, TimerMode::Repeating)
            }
            AddCooldownOnCompletion
            on(shifty_shot_activate)
            HasValidators[
                (
                    #ShiftyShotCDValidator
                    AbilityOffCooldown
                ),
                (
                    #ShiftyShotRangeValidator
                    EnemyInAttackRange
                )
            ]
        ]
    ),
    (
        #ShiftyShotEnemy1
        Enemy
        Position(e1_pos)
        Sprite {image: "enemies/faceless/sprite.png"}
        Health::new(9001.0)

    ),
    (
        #ShiftyShotEnemy2
        Enemy
        Position(e2_pos)
        Sprite {image: "enemies/faceless/sprite.png"}
        Health::new(42069.0)
        )
        ]
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum AbilityState {
    /// We're just been made
    #[default]
    Init,
    /// We've been requested to start, which can be triggered by potentially many different ways.
    ///
    /// This will reach out to the validators that the ability has in order to make sure that this
    /// is allowed to happen
    Requested,
    /// This ability has been started and is ongoing execution
    Executing,
    /// Something has happened that causes this to be cancelled
    Cancelled,
    /// This is not allowed to run for some reason
    Failure,
}

/// An individual entity that is responsible for saying whether or not this ability can proceed.
/// Validators run before systems that check whether or not something should be allowed to move from
/// Requested to Executing
#[derive(Component, Debug, Clone, Copy)]
pub struct AbilityValidator {
    pub value: bool,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct Ability {
    pub kind: AbilityKind,
    pub activates: AbilityStart,
    pub while_active: WhileAbilityActive,
    pub deactivates: AbilityDeactivation,
    times_activated: u32,
}

#[derive(Clone, Debug, Default, Reflect)]
pub enum AbilityKind {
    #[default]
    PassiveHealth,
    GameSpawnEnemies,
    PassiveHealthRegen,
    DiceGuard {
        dice: Option<Vec<Entity>>,
    },
    PaddleBack,
    ShiftyShot,
    BumpinTunes,
}

#[derive(Clone, Debug, Default, Reflect)]
/// Baseline requirwement: you can't be on cooldown, you can't be deactivated (obvs).
/// Everything else comes from here
pub enum AbilityStart {
    #[default]
    Immediately,
    /// Want to rethink this so badly, but later
    EnemiesInRange,
    PlayersInRange,
}

#[derive(Clone, Debug, Default, Reflect)]
pub enum WhileAbilityActive {
    #[default]
    DoNothing,
    PulseActivations {
        pulse: Timer,
    },
    StoreCharges {
        rem_charges: u8,
        tickrate: Timer,
    },
}

#[derive(Clone, Debug, Default, Reflect)]
pub enum AbilityDeactivation {
    #[default]
    Never,
    AfterTimer(Timer),
    AfterActivations(u32),
}

#[derive(EntityEvent, Clone, Copy, Debug, Reflect)]
pub struct ActivateAbility {
    entity: Entity,
}

#[derive(EntityEvent, Clone, Copy, Debug, Reflect)]
pub struct DeactivateAbility {
    entity: Entity,
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
pub struct AbilityActive;

#[derive(Component, Debug, Clone, Reflect)]
#[relationship_target(relationship = AbilityOf)]
pub struct HasAbilities(Vec<Entity>);

#[derive(Component, Debug, Clone, Reflect)]
#[relationship(relationship_target = HasAbilities)]
pub struct AbilityOf(Entity);

/// Run in precombat
pub fn check_ability_activation(
    mut commands: Commands,
    mut q_ability: Query<
        (
            Entity,
            &mut Ability,
            &AbilityOf,
            Option<&AttackRange>,
            Option<&EffectDuration>,
        ),
        (Without<Cooldown>, Without<AbilityActive>),
    >,
    mut set_search: ParamSet<(
        Query<&Position, With<HasAbilities>>,
        Query<&Position, With<Enemy>>,
        Query<&Position, With<Player>>,
    )>,
) {
    for (a_ent, mut ability, ability_of, m_range, m_dur) in &mut q_ability {
        let start = match ability.activates {
            AbilityStart::Immediately => true,
            AbilityStart::EnemiesInRange => {
                let q_self = set_search.p0();
                let self_pos = q_self
                    .get(ability_of.0)
                    .expect("The entity this ability was attached to doesn't have a position");
                let pos_vec = self_pos.0.clone();
                let range = m_range
                    .expect("Must have range for this ability to activate")
                    .0;
                set_search
                    .p1()
                    .iter()
                    .any(|pos| pos_vec.distance(pos.0) <= range)
            }
            AbilityStart::PlayersInRange => {
                let q_self = set_search.p0();
                let self_pos = q_self
                    .get(ability_of.0)
                    .expect("The entity this ability was attached to doesn't have a position");
                let pos_vec = self_pos.0.clone();
                let range = m_range
                    .expect("Must have range for this ability to activate")
                    .0;
                set_search
                    .p2()
                    .iter()
                    .any(|pos| pos_vec.distance(pos.0) <= range)
            }
        };

        if start {
            commands.entity(a_ent).insert(AbilityActive);
            commands.trigger(ActivateAbility { entity: a_ent });
            ability.times_activated = 1;

            match ability.deactivates {
                AbilityDeactivation::AfterTimer(ref mut timer) => {
                    let dur = m_dur.expect("No Effect Duration on After Timer Ability").0;
                    timer.set_duration(Duration::from_secs_f32(dur));
                    timer.reset();
                }
                _ => {}
            }
        }
    }
}

fn while_ability_active(
    mut commands: Commands,
    time: Res<Time>,
    mut q_ability: Query<(Entity, &mut Ability), With<AbilityActive>>,
) {
    for (a_ent, mut ability) in &mut q_ability {
        let activate = match ability.while_active {
            WhileAbilityActive::DoNothing => false,
            WhileAbilityActive::PulseActivations { ref mut pulse } => {
                pulse.tick(time.delta());
                if pulse.just_finished() {
                    pulse.reset();
                    true
                } else {
                    false
                }
            }
            WhileAbilityActive::StoreCharges {
                rem_charges: _,
                tickrate: _,
            } => false,
        };
        if activate {
            commands.trigger(ActivateAbility { entity: a_ent });
            ability.times_activated += 1;
        }
    }
}

fn check_ability_end(
    mut commands: Commands,
    time: Res<Time>,
    mut q_ability: Query<
        (Entity, &mut Ability, &AbilityOf, Option<&CooldownRate>),
        (With<AbilityActive>),
    >,
) {
    for (a_ent, mut ability, a_of, m_cdr) in &mut q_ability {
        let deactivate = match ability.deactivates {
            AbilityDeactivation::Never => false,
            AbilityDeactivation::AfterTimer(ref mut timer) => {
                timer.tick(time.delta());
                timer.just_finished()
            }
            AbilityDeactivation::AfterActivations(num) => ability.times_activated == num,
        };

        if deactivate {
            let cd = m_cdr.expect("No CooldownRate on Deactivating Ability");
            commands
                .entity(a_ent)
                .insert(Cooldown::new(cd.0))
                .remove::<AbilityActive>();
            commands.trigger(DeactivateAbility { entity: a_ent });
        }
    }
}

pub struct AbilityPlugin;
impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        // This needs to run with more specificity
        // later. But for now we're spawning this in the main menu
        app.add_systems(
            FixedUpdate,
            (
                check_ability_activation,
                while_ability_active,
                check_ability_end,
            )
                .chain()
                .run_if(in_state(InGameState::InGame))
                .in_set(CombatSystemSet::Combat),
        );
    }
}

fn generic_observer(t: On<ActivateAbility>) {
    info!("Ability Activated!")
}

fn dice_guard_demo(position: Vec2) -> impl SceneList {
    let e_pos = position + Vec2::Y * 100.0;
    bsn_list! [(
        #DummyPlayer1
        Position(position)
        HasAbilities [ (
                #DiceGuard
                AttackRange(100.0)
                EffectSize(50.0)
                EffectDuration(5.0)
                ProjectileCount(3.0)
                ProjectileSpeed(20.0)
                CooldownRate(3.0)
                Damage(5.0)
                Ability {
                    kind: AbilityKind::DiceGuard { dice: None },
                    activates: AbilityStart::Immediately,
                    while_active: WhileAbilityActive::DoNothing,
                    deactivates: AbilityDeactivation::AfterTimer(Timer::from_seconds(5.0, TimerMode::Once)),
                }
                on(generic_observer)
                on(dice_guard_activate)
                on(dice_guard_deactivate)
        )]
        ),
        (
            #DiceGuardEnemy
            Enemy
            Position(e_pos)
        )
    ]
}

pub fn shifty_shot_demo(position: Vec2) -> impl SceneList {
    let e1_pos = position + Vec2::Y * 250.0;
    let e2_pos = position + Vec2::Y * 250.0 + Vec2::X * 100.0;
    bsn_list! [(
        #DummyPlayer
        Player
        Position(position)
        Transform {translation: Vec3::new(400.0, 0.0, 0.0)}
        HasAbilities [
            #ShiftyShot
            AttackRange(250.0)
            ProjectileBounces(1.0)
            ProjectileCount(3.0)
            ProjectileSpeed(50.0)
            CooldownRate(3.0)
            CritChance(0.5)
            CritDamage(0.5)
            Damage(5.0)
            Ability {
                kind: AbilityKind::ShiftyShot,
                activates: AbilityStart::EnemiesInRange,
                while_active: WhileAbilityActive::PulseActivations { pulse: Timer::from_seconds(0.5, TimerMode::Once) },
                deactivates: AbilityDeactivation::AfterActivations(3),
            }
            on(shifty_shot_activate)
        ]
    ),
    (
        #ShiftyShotEnemy1
        Enemy
        Position(e1_pos)
        Sprite {image: "enemies/faceless/sprite.png"}
        Health::new(9001.0)

    ),
    (
        #ShiftyShotEnemy2
        Enemy
        Position(e2_pos)
        Sprite {image: "enemies/faceless/sprite.png"}
        Health::new(42069.0)
        )
        ]
}

fn bumpin_tunes_demo(position: Vec2) -> impl SceneList {
    let e1_pos = position + Vec2::Y * 250.0;
    let e2_pos = position + Vec2::X * 250.0;
    let e3_pos = position + Vec2::NEG_Y * 250.0;
    let e4_pos = position + Vec2::NEG_X * 250.0;
    bsn_list! [(
    #DummyPlayer
    Player
    Position(position)
    HasAbilities [
        #Tunes
        EffectSize(350.0)
        Damage(5.0)
        Ability {
            kind: AbilityKind::BumpinTunes,
            activates: AbilityStart::Immediately,
            while_active: WhileAbilityActive::PulseActivations { pulse: Timer::from_seconds(0.5, TimerMode::Once) },
            deactivates: AbilityDeactivation::Never,
        }
        on(bumpin_tunes_activate)
    ]
    ),
    (
    #TunesEnemy1
    Enemy
    Position(e1_pos)
    Sprite {image: "enemies/faceless/sprite.png"}
    Health::new(42069.0)
    ),
    (
    #TunesEnemy2
    Enemy
    Position(e2_pos)
    Sprite {image: "enemies/faceless/sprite.png"}
    Health::new(42069.0)
    ),
    (
    #TunesEnemy3
    Enemy
    Position(e3_pos)
    Sprite {image: "enemies/faceless/sprite.png"}
    Health::new(42069.0)
    ),
    (
    #TunesEnemy4
    Enemy
    Position(e4_pos)
    Sprite {image: "enemies/faceless/sprite.png"}
            Health::new(42069.0)
    )
    ]
}

/// This is going to do a lot of assumptions for the moment, because I think switching straight to in game would probably break some things
/// this also takes advantage of CombatSystemSet not being scoped to InGameState::InGame, which is only true because it couldn't be done at last check!
pub fn debug_launch_abilities_demo(
    mut commands: Commands,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<InGameState>>,
    q_main_menu_screen: Single<Entity, With<MainMenuScreen>>,
) {
    commands.entity(*q_main_menu_screen).despawn();
    app_state.set(AppState::InGame);
    game_state.set(InGameState::InGame);
    commands.spawn_scene_list(dice_guard_demo(Vec2::ZERO));
    commands.spawn_scene_list(shifty_shot_demo(Vec2::new(400.0, 0.0)));
    commands.spawn_scene_list(bumpin_tunes_demo(Vec2::new(800.0, 0.0)));
}

fn dice_guard_activate(
    on: On<ActivateAbility>,
    mut commands: Commands,
    game_kind: Res<CurrentGameKind>,
    mut q_dice_guard: Query<(
        &mut Ability,
        &AbilityOf,
        &ProjectileCount,
        &EffectSize,
        &ProjectileSpeed,
        &Damage,
    )>,
    q_holder: Query<&Position>,
) {
    if let Ok((mut ability, holder, p_count, eff_size, proj_speed, dam)) =
        q_dice_guard.get_mut(on.entity)
    {
        let mut projectiles = vec![];
        info!("Dice guard activated!");
        let holder_pos = q_holder.get(holder.0).unwrap();

        let iters = p_count.0.floor() as usize;

        for i in 0..iters {
            // Shorhand for now
            let r = eff_size.0 * 4.0;
            //spawn_positions.positions_2d().into_iter().enumerate() {
            let angle = std::f32::consts::TAU * (i as f32 / p_count.0);
            let proj = Projectile {
                movement: ProjectileMovement::Orbital {
                    around: holder.0,
                    speed: proj_speed.0,
                    c_angle: angle,
                    radius: r,
                },
            };
            let pos = holder_pos.0 + Vec2::from_angle(angle) * r;
            trace!("Found angle to be {angle}, position is {:?}", pos);
            let ent = spawn_game_object(
                &mut commands,
                game_kinds::GameKinds::SinglePlayer,
                //game_kind.0.unwrap(),
                None::<()>,
                MultiPlayerComponentOptions::from(proj),
                (
                    proj,
                    DiceGuardProjectile,
                    Position(pos),
                    CreatedBy(holder.0),
                    *dam,
                    *eff_size,
                    AppliesCollisionEffect::new(
                        [ColliderTypes::Enemy].into(),
                        ApplyDamage::default(),
                    ),
                ),
            );
            projectiles.push(ent);
        }
        match ability.kind {
            AbilityKind::DiceGuard { ref mut dice } => {
                *dice = Some(projectiles);
            }
            _ => {}
        }
    }
}

fn dice_guard_deactivate(
    on: On<DeactivateAbility>,
    mut commands: Commands,
    mut q_ability: Query<&mut Ability>,
) {
    if let Ok(mut ability) = q_ability.get_mut(on.entity) {
        match ability.kind {
            AbilityKind::DiceGuard { ref mut dice } => {
                if let Some(list) = dice {
                    for projectile in list.iter() {
                        commands.entity(*projectile).despawn();
                    }
                }
                *dice = None;
            }
            _ => {}
        }
    }
}

/// Right now, this only targets enemies. But we could change that in the future
fn shifty_shot_activate(
    on: On<ActivateAbility>,
    mut commands: Commands,
    mut q_shifty_shot: Query<(
        &mut Ability,
        &AbilityOf,
        &ProjectileSpeed,
        &Damage,
        &ProjectileBounces,
        &AttackRange,
        &CritChance,
        &CritDamage,
    )>,
    q_transforms: Query<(Entity, &Position, Has<Enemy>)>,
) {
    if let Ok((mut shifty_shot, holder, speed, damage, bounces, range, cc, cd)) =
        q_shifty_shot.get_mut(on.entity)
    {
        info!("Shifty Shot Activate");
        let (_, holder_pos, _) = q_transforms.get(holder.0).unwrap();

        let enemies = q_transforms
            .iter()
            .filter_map(
                |(ent, pos, enemy)| {
                    if enemy { Some((ent, pos)) } else { None }
                },
            )
            .collect::<Vec<(Entity, &Position)>>();
        let closest_enemy = find_closest_in_list(1, holder_pos.0, &enemies);
        if let Some(e) = closest_enemy.first() {
            if e.1 > range.0 {
            } else {
                let rem_bounces = bounces.0 as u8;
                let t_pos = q_transforms.get(e.0).unwrap().1;
                let init_dir = (t_pos.0 - holder_pos.0).normalize_or_zero();
                let init_vel = init_dir * speed.0;
                commands.spawn((
                    crate::shared::game_kinds::SinglePlayer,
                    ShiftyShotAttack {
                        target: Some(e.0),
                        remaining_bounces: rem_bounces,
                    },
                    *holder_pos,
                    LinearVelocity(init_vel),
                    CreatedBy(holder.0),
                    *damage,
                    *speed,
                    *range,
                    *cd,
                    *cc,
                ));
            }
        }
    }
}
fn bumpin_tunes_activate(
    trigger: On<ActivateAbility>,
    q_holder: Query<&Position, Without<Enemy>>,
    q_ability: Query<(&AbilityOf, &Damage, &EffectSize)>,
    mut q_enemies: Query<(&Position, &mut HealthBuffer), With<Enemy>>,
) {
    if let Ok((ability_of, dam, size)) = q_ability.get(trigger.entity) {
        info!("Firing bumpin tunes");
        let player_loc = q_holder
            .get(ability_of.0)
            .expect("Player position not found!");
        for (e_pos, mut buff) in &mut q_enemies {
            if player_loc.0.distance(e_pos.0) <= size.0 {
                buff.push_damage(trigger.entity, dam.0, None);
            }
        }
    }
}
