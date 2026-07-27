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
};
use serde::Deserialize;

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
