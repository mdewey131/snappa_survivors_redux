use bevy::prelude::*;
use lightyear::prelude::{AppComponentExt, PredictionRegistrationExt};
use serde::{Deserialize, Serialize};

use crate::{
    shared::{
        combat::{CombatSystemSet, Cooldown},
        game_kinds::{GameKinds, MultiPlayerComponentOptions},
        game_object_spawning::spawn_game_object,
        stats::{RawStatsList, components::*},
    },
    utils::AssetFolder,
};

mod dice_guard;
pub use dice_guard::*;

pub struct SharedWeaponPlugin;

impl Plugin for SharedWeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (weapon_off_cooldown, tick_weapon_active_timer).in_set(CombatSystemSet::Combat),
        );
    }
}

pub struct WeaponProtocolPlugin;
impl Plugin for WeaponProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Weapon>().add_prediction();
        app.register_component::<DiceGuardProjectile>()
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
        Self {
            pred: true,
            interp: false,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Reflect, Default)]
pub enum WeaponKind {
    #[default]
    DiceGuard,
    ThrowHands,
    PaddleBack,
    FlurryOfBlows,
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

pub fn add_weapon_to_player(
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
        (weapon, ChildOf(player)),
    );

    /*
    let stats: RawStatsList =
        read_ron::<RawStatsList>(format!("assets/{}/stats.ron", weapon_kind.asset_folder()));
    let mut list = StatsList::new();
    let change_events = stats.insert_into(&mut list, weapon_ent);
    */

    // Add the weapon marker components for each
    match weapon_kind {
        WeaponKind::DiceGuard => {
            commands.entity(w_ent).insert(DiceGuard);
        }
        _ => {
            todo!()
        }
    }
    w_ent
}

fn weapon_off_cooldown(
    mut commands: Commands,
    mut q_weapon: Query<
        (Entity, &mut Weapon, Option<&EffectDuration>),
        (Without<WeaponActiveTimer>, Without<Cooldown>),
    >,
) {
    for (ent, mut weapon, m_dur) in &mut q_weapon {
        match weapon.activity_pattern {
            WeaponActivityPattern::AlwaysOn => {
                commands
                    .entity(ent)
                    .insert(WeaponActiveTimer(Timer::from_seconds(
                        5.0, //**(stats.get_current(StatKind::CooldownRate).unwrap()),
                        TimerMode::Repeating,
                    )));
                commands.trigger(ActivateWeapon { entity: ent });
            }
            WeaponActivityPattern::ActiveForProjectiles {
                time_btw_attacks,
                ref mut rem_projectiles,
            } => {
                let mut proj = 4; //**(stats.get_current(StatKind::ProjectileCount).unwrap()) as u8;
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
                    *rem_projectiles -= 1;
                    commands.trigger(ActivateWeapon { entity: e });
                    if *rem_projectiles == 0 {
                        commands.entity(e).remove::<WeaponActiveTimer>();
                        commands.trigger(DeactivateWeapon { entity: e });
                    } else {
                        timer.reset()
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
