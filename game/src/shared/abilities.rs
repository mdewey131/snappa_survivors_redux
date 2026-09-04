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
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput},
    prelude::*,
    transform::commands,
    ui_widgets::Activate,
};
use bevy_egui::egui::{Key::A, epaint::text::cursor};
use bevy_enhanced_input::{action::events::Complete, condition::hold_and_release::HoldAndRelease};
use serde::{Deserialize, Serialize};

use crate::{
    client::main_menu::MainMenuScreen,
    render::animation::animate,
    shared::{
        colliders::{AppliesCollisionEffect, ApplyDamage, ColliderTypes},
        combat::{CombatSystemSet, Cooldown},
        damage::HealthBuffer,
        enemies::{Enemy, EnemyKind, spawn_enemy, spawner::EnemySpawnInstruction},
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

mod bumpin_tunes;
pub use bumpin_tunes::*;

mod dice_guard;
pub use dice_guard::*;

mod paddle_back;
pub use paddle_back::*;
mod targeter;
pub use targeter::Targeter;

mod throw_hands;
pub use throw_hands::*;

pub mod validators;
use validators::*;

pub struct AbilityPlugin;
impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BumpTunesPlugin)
            .configure_sets(
                FixedUpdate,
                (
                    AbilitySystemSet::CheckValidators,
                    AbilitySystemSet::CheckAbilities,
                    AbilitySystemSet::StateCheckingSystems,
                )
                    .chain()
                    .in_set(CombatSystemSet::Combat),
            )
            .configure_sets(
                FixedPostUpdate,
                ((
                    AbilitySystemSet::CheckDamageValidators,
                    AbilitySystemSet::ResolveAbilityState,
                )
                    .chain()
                    .in_set(CombatSystemSet::ResolveAbilities),),
            )
            .add_systems(
                Update,
                (
                    draw_targeter,
                    draw_attack_range_radius,
                    render_bump_tunes,
                    animate::<PaddleBackDamageCone>,
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    (
                        check_cooldown_validator,
                        enemy_in_attack_range,
                        attack_range_targeter,
                        check_has_charges,
                    )
                        .in_set(AbilitySystemSet::CheckValidators),
                    (
                        pulse_activation,
                        passive_ability,
                        active_for_timer,
                        request_on_click,
                        request_on_input,
                        completes_instantly,
                        charge_timer,
                        spawn_enemies,
                        (completes_instantly).chain(),
                        despawn_ability_on_completion,
                    )
                        .in_set(AbilitySystemSet::CheckAbilities),
                    (debug_tick_invuln_timer).in_set(CombatSystemSet::Combat),
                    (
                        set_auto_cast,
                        add_cooldown_on_ability_completion,
                        // This particular validator system has to run after the check abilities portion
                        check_step_completed,
                    )
                        .in_set(AbilitySystemSet::StateCheckingSystems),
                    // You have to run the `add_cd` system twice because the multi state ability does not hang
                    // around for one frame in the way that I'd need
                )
                    .run_if(in_state(InGameState::InGame)),
            )
            .add_systems(
                FixedPostUpdate,
                (
                    ability_holder_has_damage.in_set(AbilitySystemSet::CheckDamageValidators),
                    (single_stepped_ability, (multi_stepped_ability).chain())
                        .in_set(AbilitySystemSet::ResolveAbilityState),
                ),
            )
            .add_observer(activation_observer)
            .add_observer(remove_charge)
            .add_observer(damage_targets_on_activation);
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
#[relationship_target(relationship = AbilityStep, linked_spawn)]
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
        let starting_step = steps.current;
        let starting_state = *state;

        // Step through the inner steps
        alternate_inner_ability_recurse(
            &starting_step,
            &mut commands,
            &mut steps,
            &mut q_steps,
            &q_validators,
        );

        // test some logic about where we got to
        if steps.current >= steps.len() {
            let terminal_state = q_steps.get(*steps.steps.last().unwrap()).unwrap();
            match *terminal_state.0 {
                AbilityState::Executing => *state = AbilityState::Executing,
                AbilityState::Completed => *state = AbilityState::Completed,
                _ => {}
            }
        }
        if matches!(starting_state, AbilityState::Completed)
            && matches!(*state, AbilityState::Completed)
        {
            info!("Hi");
            // This is the part where we're allowed to reset
            steps.current = 0;
            for step in &steps.steps {
                let mut data = q_steps.get_mut(*step).unwrap();
                *data.0 = AbilityState::Init;
            }
            *state = AbilityState::Init;
        }
        // What happened if we stepped back?
        if steps.current < starting_step {
            // Look at current step
            let current_substate = steps.steps.get(steps.current).expect("Should exist");
            let substate = q_steps.get(*current_substate).expect("Substate not found!");

            let self_state = match *substate.0 {
                AbilityState::Init => {
                    if steps.current == 0 {
                        AbilityState::Init
                    } else {
                        AbilityState::Executing
                    }
                }
                AbilityState::Requested => {
                    if steps.current == 0 {
                        AbilityState::Requested
                    } else {
                        AbilityState::Executing
                    }
                }
                AbilityState::Executing => AbilityState::Executing,
                AbilityState::Cancelled => AbilityState::Cancelled,
                AbilityState::Failure => AbilityState::Failure,
                AbilityState::Completed => {
                    warn!("Not sure what happened here");
                    AbilityState::Completed
                }
            };

            info!(
                "We stepped back, your logic is probably borked somewhere. Prior: {}, Current: {}",
                starting_step, steps.current
            );
        }

        // In any case where the step we're currently on is not the end, we have to reset the stuff that comes after.
        // This will prevent things from hanging in a `Requested` state
        if steps.current < steps.steps.len() - 1 {
            for step in ((steps.current + 1)..=steps.steps.len() - 1) {
                let mut step_info = q_steps.get_mut(steps.steps[step]).expect("Step not found");
                *step_info.0 = AbilityState::Init
            }
        }
    }
}

/// Returns the state for the calling ability with steps, and sets the states of individual steps, traversing the
/// sequence until complete or cannot move further
/// Steps over the inner steps of the AbilityStep process.
/// It is expected that the caller of this function handles its own state based on
/// its state when this is done. No outer state knowledge inside the recursion!

fn alternate_inner_ability_recurse(
    from: &usize,
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
    let mut deactivate = false;
    let mut failed = false;

    info!(
        "Checking: {:?} with current state {:?}",
        steps.current, current_state
    );
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
                *step_info.0 = AbilityState::Init;
            }
        }
        AbilityState::Executing => {
            check_prior_for_completed = true;
            next_step_to_visit = Some(current + 1);
        }
        AbilityState::Cancelled => {
            deactivate = current == *from;
            next_step_to_visit = Some(current - 1);
        }
        AbilityState::Completed => {
            deactivate = current == *from;
            check_prior_for_completed = true;
            next_step_to_visit = Some(current + 1);
        }
        AbilityState::Failure => {
            deactivate = current == *from;
            failed = true;
            *step_info.0 = AbilityState::Init;
        }
    }
    info!(
        "Result: State {:?}, Failed: {:?}, Deactivated: {:?}",
        step_info.0, failed, deactivate
    );
    if deactivate {
        commands.trigger(DeactivateAbility { entity: *step });
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
        if c < current {
            info!("We stepped back, I bet there's a bug here!");
        }
        alternate_inner_ability_recurse(&current, commands, steps, q_steps, q_validators);
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

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct CompletesInstantly;
fn completes_instantly(mut q_ability: Query<&mut AbilityState, With<CompletesInstantly>>) {
    for mut state in &mut q_ability {
        match *state {
            AbilityState::Executing => {
                *state = AbilityState::Completed;
            }
            _ => {}
        }
    }
}

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

/// Tracks how many "charges" this thing currently has
///
/// Assumes the Cooldown rate should be used for understanding the charge rate
#[derive(Component, Debug, Clone, Default, Reflect)]
pub struct HoldsCharges {
    max: u8,
    current: u8,
    timer: Timer,
}
impl HoldsCharges {
    fn new(max: u8, duration: f32) -> Self {
        Self {
            max,
            current: 0,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

pub fn charge_timer(
    time: Res<Time<Virtual>>,
    mut q_charges: Query<(Entity, &mut HoldsCharges, Option<&AbilityStep>)>,
    q_cooldown_rates: Query<&CooldownRate>,
) {
    for (ent, mut charges, m_step) in &mut q_charges {
        if charges.current == charges.max {
            continue;
        }
        charges.timer.tick(time.delta());
        if charges.timer.just_finished() {
            charges.current += 1;
            let cdr_ent = if let Some(s) = m_step { s.step_of } else { ent };
            let cdr = q_cooldown_rates.get(cdr_ent).expect("Should exist");
            charges.timer.set_duration(Duration::from_secs_f32(cdr.0));
            charges.timer.reset()
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct RemoveChargeOnActivation;
fn remove_charge(
    on: On<ActivateAbility>,
    q_step: Query<&AbilityStep, With<RemoveChargeOnActivation>>,
    mut q_charges: Query<(&mut HoldsCharges, Option<&RemoveChargeOnActivation>)>,
) {
    let mut charges = if let Ok(step) = q_step.get(on.entity) {
        let (mut c, _) = q_charges
            .get_mut(step.step_of)
            .expect("Activated ability is a step of somehting without charges");
        c
    } else if let Ok((mut c, m_c)) = q_charges.get_mut(on.entity) {
        if m_c.is_none() {
            warn!("This thing doesn't have instruction to remove charges, but holds them?");
            return;
        }
        c
    } else {
        return;
    };
    charges.current -= 1;
}

/// If this ability is complete, use the ability's CDR to make a Cooldown Component
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct AddCooldownOnCompletion;
fn add_cooldown_on_ability_completion(
    mut commands: Commands,
    q_ability: Query<
        (Entity, &AbilityState, &CooldownRate),
        (Without<Cooldown>, With<AddCooldownOnCompletion>),
    >,
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

/// If this ability is complete, use the ability's CDR to make a Cooldown Component
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct DespawnOnCompletion;
fn despawn_ability_on_completion(
    mut commands: Commands,
    q_ability: Query<(Entity, &AbilityState, Option<&AbilityStep>), With<DespawnOnCompletion>>,
) {
    for (ent, state, m_step) in &q_ability {
        let to_despawn = if let Some(ab) = m_step {
            ab.step_of
        } else {
            ent
        };
        match state {
            AbilityState::Completed => {
                commands.entity(to_despawn).despawn();
            }
            _ => {}
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct DamageTargetsOnActivation(pub Vec<Entity>);

fn damage_targets_on_activation(
    observer: On<ActivateAbility>,
    q_triggering_ent: Query<(
        Entity,
        &AbilityState,
        Option<&AbilityStep>,
        &DamageTargetsOnActivation,
    )>,
    q_attack: Query<(&Damage, &CritChance, &CritDamage)>,
    mut q_target: Query<&mut HealthBuffer>,
) {
    if let Ok((ability_ent, state, m_step, targets)) = q_triggering_ent.get(observer.entity) {
        info!("Made it here");
        let stat_entity = if let Some(step) = m_step {
            step.step_of
        } else {
            ability_ent
        };
        if let Ok((dam, cc, cd)) = q_attack.get(stat_entity) {
            for t in &targets.0 {
                let mut hp = q_target
                    .get_mut(*t)
                    .expect("Targeting an entity without a health buffer");
                info!("Pushing damage to {:?}", t);
                hp.push_damage(stat_entity, dam.0, Some((cc.0, cd.0)));
            }
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct SpawnEnemies(pub EnemySpawnInstruction);
fn spawn_enemies(
    mut commands: Commands,
    q_spawner: Query<(&SpawnEnemies, &AbilityState)>,
    q_positions: Query<&Position, With<Player>>,
) {
    for (spawn, state) in &q_spawner {
        if matches!(*state, AbilityState::Completed) {
            let positions = spawn.0.pattern.to_positions(&q_positions);
            for position in positions {
                spawn_enemy(&mut commands, spawn.0.kind, position);
            }
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
    CheckDamageValidators,
    /// These systems reset things to their proper place using the state machinery of these abilities
    /// Note: becuase effects include writing damage, these have to run before any health checking systems.
    /// Otherwise, you create a condition where damage can get written to the health buffer while other systems are working
    /// on that buffer, which is a no-no
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
                    trace!("Timer finished!");
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
        trace!("Incrementing Activation for {}", trigger.entity);
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
    mut commands: Commands,
    mut gizmos: Gizmos,
    q_camera: Single<(&Camera, &GlobalTransform)>,
    q_window: Single<&Window>,
    mut q_targeter: Option<Single<&mut Position, With<Targeter>>>,
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
                if q_targeter.is_none() {
                    commands.spawn((Targeter, Position(cursor_position)));
                } else if let Some(ref mut t_pos) = q_targeter {
                    t_pos.0 = world_pos;
                }
                gizmos.circle_2d(world_pos, 10., bevy::color::palettes::basic::WHITE);
                // Should be the same as world_pos
                gizmos.circle_2d(world_check, 8., bevy::color::palettes::basic::RED);
            }
        }
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct DrawAttackRangeRadius;
fn draw_attack_range_radius(
    mut gizmos: Gizmos,
    q_ability: Query<
        (
            &AbilityState,
            Option<&AbilityStep>,
            Option<&AttackRange>,
            Option<&AbilityOf>,
        ),
        With<DrawAttackRangeRadius>,
    >,
    q_outer_ent: Query<(&AttackRange, &AbilityOf), Without<DrawAttackRangeRadius>>,
    q_positions: Query<&Position, With<HasAbilities>>,
) {
    for (state, m_step, m_range, m_holder) in q_ability {
        let should_draw = matches!(*state, AbilityState::Executing);
        if should_draw {
            let (attack_range, holder_position) = {
                if let Some(s) = m_step {
                    let ar = q_outer_ent.get(s.step_of).expect("Should exist").0;
                    let holder_ent = q_outer_ent.get(s.step_of).expect("Should exist").1;
                    let holder = q_positions
                        .get(holder_ent.0)
                        .expect("Entity holding this ability does not have a Position");
                    (ar, holder)
                } else {
                    let ar = m_range.expect("This ability does not have an attack range");
                    let holder_ent = m_holder.expect("Ability does not have a holder");
                    let holder = q_positions
                        .get(holder_ent.0)
                        .expect("Entity holding this ability does not have a Position");
                    (ar, holder)
                }
            };

            gizmos.circle_2d(
                holder_position.0,
                attack_range.0,
                bevy::color::palettes::basic::WHITE,
            );
        }
    }
}

// Works off of an input map, not yet created
#[derive(Component, Clone, Debug, Default)]
pub struct RequestOnInput(pub String);
pub fn request_on_input(
    input: Res<ButtonInput<KeyCode>>,
    mut q_ability: Query<(&mut AbilityState, &RequestOnInput)>,
) {
    for (mut state, req) in &mut q_ability {
        let run = matches!(*state, AbilityState::Init) & input.just_pressed(KeyCode::KeyE);
        if run {
            *state = AbilityState::Requested
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
#[relationship_target(relationship = AbilityOf, linked_spawn)]
pub struct HasAbilities(Vec<Entity>);

#[derive(Component, Debug, Clone, Reflect)]
#[relationship(relationship_target = HasAbilities)]
pub struct AbilityOf(pub Entity);

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
    /*
    #[cfg(feature = "dev")]
    commands.spawn_scene_list(dice_guard_demo(500.0 * Vec2::NEG_X));
    //#[cfg(feature = "dev")]
    //commands.spawn_scene_list(targeting_step_ability_demo(Vec2::ZERO));
    #[cfg(feature = "dev")]
    commands.spawn_scene_list(bump_tunes_demo(500.0 * Vec2::X));
    #[cfg(feature = "dev")]
    commands.spawn_scene_list(throw_hands_demo(500.0 * Vec2::Y));
    */
    #[cfg(feature = "dev")]
    commands.spawn_scene_list(paddle_back_demo(500.0 * Vec2::NEG_Y));
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
