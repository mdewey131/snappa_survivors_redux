use bevy::prelude::*;
use lightyear::prelude::AppComponentExt;
use serde::{Deserialize, Serialize};

use crate::shared::combat::{CombatSystemSet, Cooldown};

pub mod dice_guard;
use dice_guard::DiceGuard;

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
        app.register_component::<Weapon>();
    }
}

#[derive(Component, Serialize, Deserialize, Debug, PartialEq)]
pub struct Weapon {
    kind: WeaponKind,
    activity_pattern: WeaponActivityPattern,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Reflect, Default)]
pub enum WeaponKind {
    #[default]
    DiceGuard,
    ThrowHands,
    PaddleBack,
    FlurryOfBlows,
}

impl WeaponKind {
    pub fn asset_folder(&self) -> String {
        match self {
            WeaponKind::DiceGuard => "weapons/dice_guard".into(),
            _ => "unknown!".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum WeaponActivityPattern {
    AlwaysOn,
    ActiveforDuration,
    /// While active, this will tick down the rem_projectiles
    ActiveForProjectiles {
        time_btw_attacks: f32,
        rem_projectiles: u8,
    },
}

impl From<WeaponKind> for WeaponActivityPattern {
    fn from(value: WeaponKind) -> Self {
        match value {
            WeaponKind::DiceGuard => WeaponActivityPattern::ActiveforDuration,
            _ => WeaponActivityPattern::AlwaysOn,
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

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct WeaponActiveTimer(pub Timer);

pub fn add_weapon_to_player(
    player: Entity,
    commands: &mut Commands,
    weapon_kind: WeaponKind,
) -> Entity {
    info!("Adding Weapon to target");
    let weapon_ent = commands.spawn_empty().id();
    let pattern: WeaponActivityPattern = weapon_kind.into();
    let weapon = Weapon {
        kind: weapon_kind,
        activity_pattern: pattern,
    };
    /*
    let stats: RawStatsList =
        read_ron::<RawStatsList>(format!("assets/{}/stats.ron", weapon_kind.asset_folder()));
    let mut list = StatsList::new();
    let change_events = stats.insert_into(&mut list, weapon_ent);
    */
    let mut w_com = commands.entity(weapon_ent);
    w_com.insert((weapon /*list*/,));

    // Add the weapon marker components for each
    match weapon_kind {
        WeaponKind::DiceGuard => {
            w_com.insert(DiceGuard);
        }
        _ => {
            todo!()
        }
    }

    commands.entity(player).add_child(weapon_ent);

    /*     commands.queue(move |world: &mut World| {
        world.write_message_batch(change_events);
    });
    */
    weapon_ent
}

fn weapon_off_cooldown(
    mut commands: Commands,
    mut q_weapon: Query<
        (Entity, &mut Weapon /*&StatsList*/),
        (Without<WeaponActiveTimer>, Without<Cooldown>),
    >,
) {
    for (ent, mut weapon /*stats*/) in &mut q_weapon {
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
                let dur = 5.0; //**(stats.get_current(StatKind::EffectDuration).unwrap());
                commands.trigger(ActivateWeapon { entity: ent });
                commands
                    .entity(ent)
                    .insert(WeaponActiveTimer(Timer::from_seconds(dur, TimerMode::Once)));
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
