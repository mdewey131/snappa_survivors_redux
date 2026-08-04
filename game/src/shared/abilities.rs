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
    input::mouse::MouseButtonInput,
    prelude::*,
    transform::commands,
    ui_widgets::Activate,
};
use serde::{Deserialize, Serialize};

use crate::{
    client::main_menu::MainMenuScreen,
    shared::{
        colliders::{AppliesCollisionEffect, ApplyDamage, ColliderTypes},
        combat::{CombatSystemSet, Cooldown},
        damage::HealthBuffer,
        enemies::Enemy,
        game_kinds::{
            self, CurrentGameKind,
            GameKinds::{self, SinglePlayer},
            MultiPlayerComponentOptions,
        },
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

#[cfg(feature = "dev")]
pub mod demo;
#[cfg(feature = "dev")]
use demo::*;

pub mod targeter;
use targeter::Targeter;

pub mod validators;
use validators::*;

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
                (
                    check_cooldown_validator,
                    enemy_in_attack_range,
                    attack_range_targeter,
                )
                    .in_set(AbilitySystemSet::CheckValidators),
                (
                    pulse_activation,
                    passive_ability,
                    active_for_timer,
                    request_on_click,
                    draw_targeter,
                )
                    .in_set(AbilitySystemSet::CheckAbilities),
                (set_auto_cast, add_cooldown_on_ability_completion)
                    .in_set(AbilitySystemSet::StateCheckingSystems),
                // You have to run the `add_cd` system twice because the multi state ability does not hang
                // around for one frame in the way that I'd need
                (
                    single_stepped_ability,
                    (multi_stepped_ability, add_cooldown_on_ability_completion).chain(),
                )
                    .in_set(AbilitySystemSet::ResolveAbilityState),
            )
                .run_if(in_state(InGameState::InGame)),
        )
        .add_observer(activation_observer);
    }
}

#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq, Reflect)]
pub enum AbilityState {
    #[default]
    Init,
    Requested,
    /// This step is executing. Will want to think about how starting works
    Executing,
    /// Imagine a channeled ability gets cancelled, or a tether breaks, etc
    Cancelled,
    /// Set by the component for the ability itself
    Completed,
    /// This wasn't allowed to execute
    Failure,
}

#[derive(Component, Debug, Clone, Default, Reflect)]
#[require(AbilityState = AbilityState::default())]
pub struct Ability;

#[derive(Component, Debug, Clone, Reflect, Default)]
#[relationship_target(relationship = AbilityStep)]
#[require(Ability = Ability)]
pub struct HasAbilitySteps {
    pub current: usize,
    pub prevent_recursion: bool,
    pub cancel_all_on_cancel_any: bool,
    #[relationship]
    steps: Vec<Entity>,
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[require(Ability = Ability)]
#[relationship(relationship_target = HasAbilitySteps)]
pub struct AbilityStep {
    #[relationship]
    pub step_of: Entity,
}

fn single_stepped_ability(
    mut commands: Commands,
    mut q_ability: Query<
        (
            Entity,
            &mut AbilityState,
            Option<&HasValidators>,
            Option<&TriggerStartAbility>,
            Option<&TriggerEndAbility>,
        ),
        (Without<HasAbilitySteps>, Without<AbilityStep>),
    >,
    q_validators: Query<&AbilityValidator>,
) {
    for (a_ent, mut state, m_validators, m_start, m_end) in &mut q_ability {
        match *state {
            AbilityState::Init => {}
            AbilityState::Requested => {
                let all_validations_true = if let Some(v) = m_validators {
                    v.iter()
                        .all(|ent| q_validators.get(ent).expect("Wat").value)
                } else {
                    true
                };
                if all_validations_true {
                    *state = AbilityState::Executing;
                    commands.trigger(ActivateAbility { entity: a_ent });
                } else {
                    *state = AbilityState::Init;
                }
            }
            AbilityState::Executing => {}
            AbilityState::Completed | AbilityState::Failure | AbilityState::Cancelled => {
                commands.trigger(DeactivateAbility { entity: a_ent });
                *state = AbilityState::Init
            }
        }
    }
}

fn multi_stepped_ability(
    mut commands: Commands,
    mut q_ability: Query<(Entity, &mut AbilityState, &mut HasAbilitySteps), Without<AbilityStep>>,
    mut q_steps: Query<(&mut AbilityState, &AbilityStep, Option<&HasValidators>)>,
    q_validators: Query<&AbilityValidator>,
) {
    for (a_ent, mut state, mut steps) in &mut q_ability {
        // Getting sick of the recursive stuff breaking my brain. Changing the rules for how this works now
        let next_ability_state =
            inner_step_recurse(&mut commands, &mut steps, &mut q_steps, &q_validators);
        info!(
            "Prior step: {:?}, current step {:?}",
            *state, next_ability_state
        );
        // This part ensures that we have at least one frame where we stay at failure/completion/cancellation
        *state = match next_ability_state {
            AbilityState::Cancelled | AbilityState::Failure | AbilityState::Completed => {
                if next_ability_state == *state {
                    info!("Deactivate!");
                    commands.trigger(DeactivateAbility { entity: a_ent });
                    // Reset all steps
                    for s in &steps.steps {
                        if let Ok((mut state, _step, _v)) = q_steps.get_mut(*s) {
                            *state = AbilityState::Init
                        }
                    }
                    steps.current = 0;
                    AbilityState::Init
                } else {
                    next_ability_state
                }
            }
            AbilityState::Executing => {
                if *state != AbilityState::Executing {
                    commands.trigger(ActivateAbility { entity: a_ent });
                }
                next_ability_state
            }
            _ => next_ability_state,
        };
    }
}

/// Returns the state for the calling ability with steps, and sets the states of individual steps, traversing the
/// sequence until complete or cannot move further
fn inner_step_recurse(
    commands: &mut Commands,
    mut steps: &mut HasAbilitySteps,
    mut q_steps: &mut Query<(&mut AbilityState, &AbilityStep, Option<&HasValidators>)>,
    q_validators: &Query<&AbilityValidator>,
) -> AbilityState {
    let current_step = steps.current;
    let current_action_step = steps.steps[current_step];
    let (mut current_state, step, m_valid) =
        q_steps.get_mut(current_action_step).expect("Where step");
    let c_state = *current_state;
    let mut moved_to_executed = false;
    let next_state_this_step = match c_state {
        AbilityState::Init => {
            info!("Came in as Init");
            AbilityState::Init
        }
        AbilityState::Requested => {
            let all_validators_true = if let Some(v) = m_valid {
                v.iter().all(|ent| q_validators.get(ent).unwrap().value)
            } else {
                true
            };

            if all_validators_true {
                commands.trigger(ActivateAbility {
                    entity: current_action_step,
                });
                moved_to_executed = true;
                AbilityState::Executing
            } else {
                info!("Hi, I'm init");
                AbilityState::Init
            }
        }
        AbilityState::Executing => AbilityState::Executing,
        AbilityState::Completed => AbilityState::Completed,
        AbilityState::Failure => AbilityState::Failure,
        AbilityState::Cancelled => AbilityState::Cancelled,
    };
    info!("Set this step to {:?}", next_state_this_step);
    let (next_step, early_return_value) = match next_state_this_step {
        AbilityState::Init => {
            if steps.current == 0 {
                (None, Some(AbilityState::Init))
            } else {
                (None, Some(AbilityState::Executing))
            }
        }
        AbilityState::Requested => {
            if steps.current == 0 {
                (None, Some(AbilityState::Requested))
            } else {
                (None, Some(AbilityState::Executing))
            }
        }
        AbilityState::Executing => {
            if !steps.prevent_recursion {
                (Some(steps.current + 1), None)
            } else {
                (None, Some(AbilityState::Executing))
            }
        }
        AbilityState::Cancelled => {
            commands.trigger(DeactivateAbility {
                entity: current_action_step,
            });
            if steps.cancel_all_on_cancel_any {
                (Some(steps.current - 1), None)
            } else {
                (Some(0), Some(AbilityState::Cancelled))
            }
        }
        AbilityState::Failure => {
            commands.trigger(DeactivateAbility {
                entity: current_action_step,
            });
            (Some(0), Some(AbilityState::Failure))
        }
        AbilityState::Completed => {
            commands.trigger(DeactivateAbility {
                entity: current_action_step,
            });
            if steps.current == (steps.steps.len() - 1) {
                (Some(0), Some(AbilityState::Completed))
            } else {
                (Some(steps.current + 1), None)
            }
        }
    };

    info!(
        "Next step to visit {:?}. Early return value {:?}",
        next_step, early_return_value
    );
    *current_state = next_state_this_step;

    // A fix to the prio value to set it to complete, in the event that this step is completed
    if matches!(next_state_this_step, AbilityState::Completed) {
        let (mut state, _step, _validators) = q_steps
            .get_mut(*steps.steps.get(current_step - 1).expect("Out of range"))
            .expect("Step not found?");
        *state = AbilityState::Completed
    }

    if let Some(v) = next_step {
        // If you're here, then you've reached the end of the line, but you need to see
        // if the state of the terminal step is completed or executing to know whether or not to return
        if v == steps.steps.len() {
            match next_state_this_step {
                AbilityState::Executing => return AbilityState::Executing,
                AbilityState::Completed => return AbilityState::Completed,
                _ => {}
            }
        } else {
            // If we're moving back, we have to reset every step between here and there to Init
            if steps.current > v {
                for i in ((v + 1)..=(current_step)) {
                    let (mut state, _step, _validators) = q_steps
                        .get_mut(*steps.steps.get(i).expect("Out of range"))
                        .expect("Step not found?");
                    *state = AbilityState::Init;
                }
            }

            steps.current = v;
        }
    }
    // We just want to do some housekeeping on the prior ability in the event that this one is executing
    if moved_to_executed {
        info!("Moved to executed this frame, setting prior to 'Completed'");
        let prior_step = current_step - 1;
        if let Some(s) = steps.steps.get(prior_step) {
            let (mut state, _, _) = q_steps.get_mut(*s).expect("Not found");
            *state = AbilityState::Completed
        }
    }

    // We have to reset what comes after, just in case
    if next_step.is_none() {
        for i in ((steps.current)..steps.steps.len()) {
            let (mut state, _step, _val) = q_steps
                .get_mut(*steps.steps.get(i).expect("Out of range"))
                .expect("Step not found");
            *state = AbilityState::Init;
        }
    }
    if let Some(s) = early_return_value {
        return s;
    } else {
        info!("Recurse!");
        return inner_step_recurse(commands, steps, q_steps, q_validators);
    }
}

/// Steps over the inner steps of the AbilityStep process.
/// It is expected that the caller of this function handles its own state based on
/// its state when this is done. No outer state knowledge inside the recursion!
fn alternate_inner_ability_recurse(
    mut commands: &mut Commands,
    mut steps: &mut HasAbilitySteps,
    mut q_steps: &mut Query<(&mut AbilityState, &AbilityStep, Option<&HasValidators>)>,
    q_validators: &Query<&AbilityValidator>,
) {
    let current = steps.current;
    let current_step = steps.steps.get(current);
    if current_step.is_none() {
        return;
    }
    let step = current_step.unwrap();
    let mut step_info = q_steps.get_mut(*step).expect("Where step?");
    let current_state = *step_info.0;
    let mut next_step_to_visit = None;
    let mut check_prior_for_completed = false;
    let mut failed = false;

    match current_state {
        AbilityState::Init => {}
        AbilityState::Requested => {
            let all_validators_true = if let Some(v) = step_info.2 {
                v.iter().all(|ent| q_validators.get(ent).unwrap().value)
            } else {
                true
            };
            if all_validators_true {
                commands.trigger(ActivateAbility { entity: *step });
                *step_info.0 = AbilityState::Executing;
                next_step_to_visit = Some(current + 1);
            } else {
                *step_info.0 = AbilityState::Failure;
            }
        }
        AbilityState::Executing => {
            check_prior_for_completed = true;
            next_step_to_visit = Some(current + 1);
        }
        AbilityState::Cancelled => {
            next_step_to_visit = Some(current + 1);
        }
        AbilityState::Completed => {
            next_step_to_visit = Some(current + 1);
        }
        AbilityState::Failure => {
            failed = true;
        }
    }
    if failed {
        return;
    }

    if check_prior_for_completed {
        let prior_step = steps.steps.get(current - 1);
        if let Some(p) = prior_step {
            let mut step_info = q_steps.get_mut(*p).expect("No Step?");
            *step_info.0 = AbilityState::Completed;
        }
    }

    if let Some(c) = next_step_to_visit {
        steps.current = c;
        alternate_inner_ability_recurse(commands, steps, q_steps, q_validators);
    } else {
        return;
    }
}

/// This ability should fire `StartAbility` when it moves from requested to executing
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct TriggerStartAbility;

/// This ability should fire `EndAbility` when it moves from requested to executing
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct TriggerEndAbility;

#[derive(Component, Debug, Default)]
pub struct CompletesInstantly;

/// I will automatically move from Init -> Requested
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct AutoCast;
fn set_auto_cast(mut q_ability: Query<&mut AbilityState, With<AutoCast>>) {
    for mut state in &mut q_ability {
        match *state {
            AbilityState::Init => *state = AbilityState::Requested,
            _ => {}
        }
    }
}

/// I am always executing
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PassiveAbility;
fn passive_ability(mut q_ability: Query<&mut AbilityState, With<PassiveAbility>>) {
    for mut state in &mut q_ability {
        *state = AbilityState::Executing
    }
}

/// If this ability is complete, use the ability's CDR to make a Cooldown Component
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct AddCooldownOnCompletion;
fn add_cooldown_on_ability_completion(
    mut commands: Commands,
    q_ability: Query<(Entity, &AbilityState, &CooldownRate), Without<Cooldown>>,
) {
    for (a_ent, state, cdr) in &q_ability {
        match state {
            AbilityState::Completed => {
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

// This will be active for the time in timer, then move to completed
#[derive(Component, Debug, Clone, Default, Reflect)]
pub struct ActiveForTime(Timer);

fn active_for_timer(
    time: Res<Time>,
    mut q_ability: Query<(&mut AbilityState, &mut ActiveForTime)>,
) {
    for (mut state, mut timer) in &mut q_ability {
        match *state {
            AbilityState::Executing => {
                timer.0.tick(time.delta());
                if timer.0.just_finished() {
                    info!("Timer finished!");
                    *state = AbilityState::Completed;
                }
            }
            AbilityState::Cancelled | AbilityState::Completed | AbilityState::Failure => {
                timer.0.reset()
            }
            _ => {}
        }
    }
}

/// Put on an ability that wants to track how many times its activated.
///
/// `PulseActivation requires this`
#[derive(Component, Debug, Clone, Default, Reflect)]
#[require(
    TriggerStartAbility = TriggerStartAbility,
)]
pub struct Activations {
    pub current: u32,
    pub max: Option<u32>,
}
// This will be active for the entire duration, and send ActivateAbility messages
// when the timer is up.
#[derive(Component, Debug, Clone, Default, Reflect)]
#[require(
    Activations = Activations::default(),
)]
pub struct PulseActivation {
    pub timer: Timer,
}

fn pulse_activation(
    mut commands: Commands,
    time: Res<Time>,
    mut q_pulse: Query<(
        Entity,
        &AbilityState,
        &mut PulseActivation,
        &mut Activations,
        Option<&ProjectileCount>,
    )>,
) {
    for (a_ent, state, mut pulse, mut activations, m_p_count) in &mut q_pulse {
        match state {
            AbilityState::Init => {
                let next_pulse_count = m_p_count.map(|proj_count| proj_count.0 as u32);
                activations.max = next_pulse_count;
                activations.current = 0;
            }
            AbilityState::Executing => {
                pulse.timer.tick(time.delta());
                if pulse.timer.just_finished() {
                    pulse.timer.reset();
                    commands.trigger(ActivateAbility { entity: a_ent });
                }
            }
            _ => {}
        }
    }
}
fn activation_observer(
    trigger: On<ActivateAbility>,
    mut q_activations: Query<(&mut Activations, &mut AbilityState)>,
) {
    if let Ok((mut activations, mut state)) = q_activations.get_mut(trigger.entity) {
        info!("Incrementing Activation for {}", trigger.entity);
        activations.current += 1;
        if let Some(m) = activations.max {
            if m == activations.current {
                *state = AbilityState::Completed
            }
        }
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct DrawTargeterOnMouse;
fn draw_targeter(
    mut gizmos: Gizmos,
    q_camera: Single<(&Camera, &GlobalTransform)>,
    q_window: Single<&Window>,
    mut q_targeter: Query<&mut Position, With<Targeter>>,
    q_ability: Query<&AbilityState, With<DrawTargeterOnMouse>>,
) {
    for state in q_ability {
        let should_draw = matches!(*state, AbilityState::Executing);
        if should_draw {
            let (camera, camera_transform) = *q_camera;
            if let Some(cursor_position) = q_window.cursor_position()
                    // Calculate a world position based on the cursor's position.
                    && let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position)
                    // To test Camera::world_to_viewport, convert result back to viewport space and then back to world space.
                    && let Ok(viewport_check) = camera.world_to_viewport(camera_transform, world_pos.extend(0.0))
                    && let Ok(world_check) = camera.viewport_to_world_2d(camera_transform, viewport_check.xy())
            {
                gizmos.circle_2d(world_pos, 10., bevy::color::palettes::basic::WHITE);
                // Should be the same as world_pos
                gizmos.circle_2d(world_check, 8., bevy::color::palettes::basic::RED);
            }
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
                Ability
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
        HasAbilities [
            #ShiftyShot
            AttackRange(250.0)
            ProjectileBounces(1.0)
            ProjectileCount(3.0)
            ProjectileSpeed(100.0)
            CooldownRate(3.0)
            CritChance(0.5)
            CritDamage(0.5)
            Damage(5.0)
            Ability
            AutoCast
            PulseActivation {
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

fn bumpin_tunes_demo(position: Vec2) -> impl SceneList {
    let e1_pos = position + Vec2::Y * 100.0;
    let e2_pos = position + Vec2::X * 100.0;
    let e3_pos = position + Vec2::NEG_Y * 100.0;
    let e4_pos = position + Vec2::NEG_X * 100.0;
    bsn_list! [(
    #DummyPlayer
    Player
    Position(position)
    HasAbilities [
        #Tunes
        AutoCast
        EffectSize(350.0)
        Damage(5.0)
        Ability
        TriggerStartAbility
        PulseActivation {timer: Timer::from_seconds(0.75, TimerMode::Repeating)}
        on(bumpin_tunes_activate)
        on(generic_observer)
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

#[derive(EntityEvent, Clone, Copy, Debug, Reflect)]
pub struct ActivateAbility {
    entity: Entity,
}

#[derive(EntityEvent, Clone, Copy, Debug, Reflect)]
pub struct DeactivateAbility {
    entity: Entity,
}

#[derive(Component, Debug, Clone, Reflect)]
#[relationship_target(relationship = AbilityOf)]
pub struct HasAbilities(Vec<Entity>);

#[derive(Component, Debug, Clone, Reflect)]
#[relationship(relationship_target = HasAbilities)]
pub struct AbilityOf(Entity);

fn generic_observer(t: On<ActivateAbility>) {
    info!("Ability Activated!")
}

/*
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
*/

/// This is going to do a lot of assumptions for the moment, because I think switching straight to in game would probably break some things
/// this also takes advantage of CombatSystemSet not being scoped to InGameState::InGame, which is only true because it couldn't be done at last check!
pub fn debug_launch_abilities_demo(
    mut commands: Commands,
    mut game_kind: ResMut<CurrentGameKind>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<InGameState>>,
    q_main_menu_screen: Single<Entity, With<MainMenuScreen>>,
) {
    commands.entity(*q_main_menu_screen).despawn();
    app_state.set(AppState::InGame);
    game_state.set(InGameState::InGame);
    game_kind.0 = Some(GameKinds::SinglePlayer);
    #[cfg(feature = "dev")]
    commands.spawn_scene_list(targeting_step_ability_demo());
    /*
    commands.spawn_scene_list(new_dice_guard_scene());
    commands.spawn_scene_list(new_shifty_shot_scene(Vec2::X * 500.0));
    commands.spawn_scene_list(bumpin_tunes_demo(Vec2::NEG_X * 500.0));
    */
    /*
    commands.spawn_scene_list(dice_guard_demo(Vec2::ZERO));
    commands.spawn_scene_list(shifty_shot_demo(Vec2::new(400.0, 0.0)));
    commands.spawn_scene_list(bumpin_tunes_demo(Vec2::new(800.0, 0.0)));
     */
}

#[derive(Component, Clone, Debug, Default)]
pub struct DiceGuard {
    pub dice: Option<Vec<Entity>>,
}
fn dice_guard_activate(
    on: On<ActivateAbility>,
    mut commands: Commands,
    game_kind: Res<CurrentGameKind>,
    mut q_dice_guard: Query<(
        &mut DiceGuard,
        &AbilityOf,
        &ProjectileCount,
        &EffectSize,
        &ProjectileSpeed,
        &Damage,
    )>,
    q_holder: Query<&Position>,
) {
    if let Ok((mut dg, holder, p_count, eff_size, proj_speed, dam)) =
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
        dg.dice = Some(projectiles)
    }
}

fn dice_guard_deactivate(
    on: On<DeactivateAbility>,
    mut commands: Commands,
    mut q_ability: Query<&mut DiceGuard>,
) {
    if let Ok(mut dg) = q_ability.get_mut(on.entity) {
        if let Some(ref list) = dg.dice {
            for projectile in list.iter() {
                commands.entity(*projectile).despawn();
            }
        }
        dg.dice = None;
    }
}
/// If I see a click, I'm going to move my state to `Requested`.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct RequestOnClick;
fn request_on_click(
    input: Res<ButtonInput<MouseButton>>,
    mut q_ability: Query<&mut AbilityState, With<RequestOnClick>>,
) {
    if input.just_pressed(MouseButton::Left) {
        for (mut state) in &mut q_ability {
            if matches!(*state, AbilityState::Init) {
                info!("Button clicked!");
                *state = AbilityState::Requested;
            }
        }
    }
}

/// Right now, this only targets enemies. But we could change that in the future
fn shifty_shot_activate(
    on: On<ActivateAbility>,
    mut commands: Commands,
    mut q_shifty_shot: Query<(
        &mut PulseActivation,
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
    if let Ok((_ability, holder, speed, damage, bounces, range, cc, cd)) =
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
                buff.push_damage(ability_of.0, dam.0, None);
            }
        }
    }
}
