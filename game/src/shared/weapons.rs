use avian2d::prelude::Position;
use bevy::prelude::*;
use lightyear::prelude::{AppComponentExt, PredictionRegistrationExt};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{
    shared::{
        combat::{CombatSystemSet, Cooldown},
        enemies::Enemy,
        game_kinds::{GameKinds, MultiPlayerComponentOptions},
        game_object_spawning::spawn_game_object,
        players::{Player, PlayerWeapons},
        states::InGameState,
        stats::{RawStatsList, components::*},
        upgrades::PlayerUpgradeSlots,
        weapons::bumpin_tunes::BumpinTunes,
    },
    utils::AssetFolder,
};

pub mod bouncing_dice;
pub mod bumpin_tunes;
pub mod dice_guard;
pub mod shifty_shot;
pub mod throw_hands;

pub use bouncing_dice::*;
pub use dice_guard::*;
pub use shifty_shot::*;
pub use throw_hands::*;

pub struct SharedWeaponPlugin;

impl Plugin for SharedWeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (weapon_off_cooldown, tick_weapon_active_timer)
                .in_set(CombatSystemSet::Combat)
                .run_if(in_state(InGameState::InGame)),
        );
    }
}

pub struct WeaponProtocolPlugin;
impl Plugin for WeaponProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ThrowHandsProtocolPlugin, ShiftyShotProtocolPlugin));
        app.register_component::<Weapon>().add_prediction();
        app.register_component::<DiceGuardProjectile>()
            .add_prediction();
        app.register_component::<BouncingDiceAttack>()
            .add_prediction();
    }
}

#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Reflect, Clone, Copy)]
pub struct Weapon {
    kind: WeaponKind,
    activity_pattern: WeaponActivityPattern,
}

impl From<Weapon> for MultiPlayerComponentOptions {
    fn from(value: Weapon) -> Self {
        Self::PREDICTED
    }
}

#[derive(
    Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Reflect, Default, Hash, Eq, EnumIter,
)]
#[reflect(Default)]
pub enum WeaponKind {
    #[default]
    DiceGuard,
    ThrowHands,
    PaddleBack,
    FlurryOfBlows,
    BouncingDice,
    BumpinTunes,
    ShiftyShot,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Reflect, Clone, Copy)]
pub enum WeaponActivityPattern {
    AlwaysOn,
    ActiveforDuration,
    /// While active, this will tick down the rem_projectiles
    ActiveForProjectiles {
        time_btw_attacks: f32,
        rem_projectiles: u8,
    },
}

impl From<WeaponKind> for Weapon {
    fn from(value: WeaponKind) -> Self {
        let pattern = match value {
            WeaponKind::DiceGuard => WeaponActivityPattern::ActiveforDuration,
            WeaponKind::ThrowHands => WeaponActivityPattern::ActiveForProjectiles {
                time_btw_attacks: 0.5,
                rem_projectiles: 0,
            },
            WeaponKind::BouncingDice => WeaponActivityPattern::ActiveForProjectiles {
                time_btw_attacks: 0.5,
                rem_projectiles: 0,
            },
            WeaponKind::ShiftyShot => WeaponActivityPattern::ActiveForProjectiles {
                time_btw_attacks: 0.5,
                rem_projectiles: 0,
            },
            _ => WeaponActivityPattern::AlwaysOn,
        };
        Weapon {
            activity_pattern: pattern,
            kind: value,
        }
    }
}

impl From<WeaponKind> for AssetFolder {
    fn from(value: WeaponKind) -> Self {
        match value {
            WeaponKind::DiceGuard => Self("weapons/dice_guard".into()),
            WeaponKind::ThrowHands => Self("weapons/throw_hands".into()),
            WeaponKind::FlurryOfBlows => Self("weapons/flurry_of_blows".into()),
            WeaponKind::PaddleBack => Self("weapons/paddle_back".into()),
            WeaponKind::BouncingDice => Self("weapons/bouncing_dice".into()),
            WeaponKind::BumpinTunes => Self("weapons/bumpin_tunes".into()),
            WeaponKind::ShiftyShot => Self("weapons/shifty_shot".into()),
            _ => Self("unknown!".into()),
        }
    }
}

/// Triggers according to `WeaponActivityPattern`
/// this is the main event that is used to determine when weapons are supposed to do the things that they do
/// it's defined with this generic event so that every weapon can have its own activation pattern, while
/// letting the common weapon machinery handle things like cooldowns/effect duration, which apply to
/// all of them
#[derive(EntityEvent)]
pub struct ActivateWeapon {
    entity: Entity,
}

/// Triggers according to `WeaponActivityPattern`
#[derive(EntityEvent)]
pub struct DeactivateWeapon {
    entity: Entity,
}

/// Attach this component on the weapon on the client and server separately,
/// because you can't send Timers over the network
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct WeaponActiveTimer(pub Timer);

pub fn add_weapon_to_character(
    player: Entity,
    weapon_kind: WeaponKind,
    commands: &mut Commands,
    game_kind: GameKinds,
) -> Entity {
    info!("Adding Weapon to target");

    let weapon: Weapon = weapon_kind.into();
    let w_ent = spawn_game_object(
        commands,
        game_kind,
        Some(weapon_kind),
        MultiPlayerComponentOptions::from(weapon),
        (weapon, ChildOf(player), Cooldown::new(0.5)),
    );

    // Add the weapon marker components for each
    match weapon_kind {
        WeaponKind::DiceGuard => {
            commands.entity(w_ent).insert(DiceGuard);
        }
        WeaponKind::ThrowHands => {
            commands.entity(w_ent).insert(ThrowHands {
                targets: None,
                current: 0,
            });
        }
        WeaponKind::BouncingDice => {
            commands.entity(w_ent).insert(WeaponBouncingDice);
        }
        WeaponKind::BumpinTunes => {
            commands.entity(w_ent).insert(BumpinTunes);
        }
        WeaponKind::ShiftyShot => {
            commands.entity(w_ent).insert(WeaponShiftyShot);
        }
        _ => {
            warn!("Weapon entity created without a marker component")
        }
    }
    // insert this weapon into the player's weapons storage
    commands.queue(move |world: &mut World| {
        let mut q_storage = world.query::<&mut PlayerWeapons>();
        let mut player_storage = q_storage.get_mut(world, player).unwrap();
        player_storage.0.insert(weapon_kind, w_ent);

        // Special case where this is the first weapon for the player: we have
        // to add this to their upgrade slots, because there hasn't been
        // a level up event for each player, and I don't want to go the route
        // of devising a system where they level up on init
        let mut q_upgrades = world.query::<&mut PlayerUpgradeSlots>();
        let mut slots = q_upgrades.get_mut(world, player).unwrap();
        if slots.weapons.is_empty() {
            slots.weapons.insert(weapon_kind, 1);
        }
    });

    w_ent
}

fn weapon_off_cooldown(
    mut commands: Commands,
    mut q_weapon: Query<
        (
            Entity,
            &mut Weapon,
            Option<&EffectDuration>,
            Option<&ProjectileCount>,
            Option<&CooldownRate>,
        ),
        (Without<WeaponActiveTimer>, Without<Cooldown>),
    >,
) {
    for (ent, mut weapon, m_dur, m_proj_c, m_cd) in &mut q_weapon {
        match weapon.activity_pattern {
            WeaponActivityPattern::AlwaysOn => {
                commands
                    .entity(ent)
                    .insert(WeaponActiveTimer(Timer::from_seconds(
                        m_cd.expect("weapon should have cd").0,
                        TimerMode::Repeating,
                    )));
                commands.trigger(ActivateWeapon { entity: ent });
            }
            WeaponActivityPattern::ActiveForProjectiles {
                time_btw_attacks,
                ref mut rem_projectiles,
            } => {
                let mut proj = m_proj_c.unwrap().0 as u8;
                if proj > 1 {
                    proj -= 1;
                    *rem_projectiles = proj;
                    commands
                        .entity(ent)
                        .insert(WeaponActiveTimer(Timer::from_seconds(
                            time_btw_attacks,
                            TimerMode::Repeating,
                        )));
                    commands.trigger(ActivateWeapon { entity: ent });
                } else {
                    commands.trigger(ActivateWeapon { entity: ent });
                    commands.trigger(DeactivateWeapon { entity: ent });
                }
            }
            WeaponActivityPattern::ActiveforDuration => {
                commands.trigger(ActivateWeapon { entity: ent });
                commands
                    .entity(ent)
                    .insert(WeaponActiveTimer(Timer::from_seconds(
                        m_dur
                            .expect("Active duration weapon without an effect duration")
                            .0,
                        TimerMode::Once,
                    )));
            }
        }
    }
}

fn tick_weapon_active_timer(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut q_timer: Query<(Entity, &mut Weapon, &mut WeaponActiveTimer)>,
) {
    for (e, mut weapon, mut timer) in &mut q_timer {
        timer.tick(time.delta());
        if timer.just_finished() {
            match weapon.activity_pattern {
                WeaponActivityPattern::ActiveForProjectiles {
                    time_btw_attacks: _,
                    ref mut rem_projectiles,
                } => {
                    if *rem_projectiles > 0 {
                        *rem_projectiles -= 1;
                        commands.trigger(ActivateWeapon { entity: e });
                        timer.reset()
                    } else {
                        commands.entity(e).remove::<WeaponActiveTimer>();
                        commands.trigger(DeactivateWeapon { entity: e });
                    }
                }
                // This gets activated off of cooldown and so is handled by the update function
                WeaponActivityPattern::ActiveforDuration => {
                    commands.entity(e).remove::<WeaponActiveTimer>();
                    commands.trigger(DeactivateWeapon { entity: e });
                }
                WeaponActivityPattern::AlwaysOn => {
                    timer.reset();
                    commands.trigger(ActivateWeapon { entity: e });
                }
            }
        }
    }
}

pub fn find_closest_enemy_targets_to_position(
    num_to_find: u8,
    player_pos: Vec2,
    q_enemy_pos: &Vec<(Entity, &Position)>,
) -> Vec<(Entity, f32)> {
    let mut sorted = q_enemy_pos
        .iter()
        .map(|(ent, pos)| {
            let dist = pos.0.distance(player_pos);
            (*ent, *pos, dist)
        })
        .collect::<Vec<(Entity, &Position, f32)>>();

    sorted.sort_by(|q1, q2| q1.2.partial_cmp(&q2.2).unwrap());
    sorted.reverse();
    let mut targets = Vec::new();
    for _i in (0..num_to_find) {
        let m_enemy = sorted.pop();
        if let Some(record) = m_enemy {
            targets.push((record.0, record.2))
        }
    }
    targets
}
