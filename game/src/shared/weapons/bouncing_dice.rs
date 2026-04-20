use std::f32::consts::TAU;

use super::*;
use crate::shared::{
    combat::{CharacterFacing, FacingDirection},
    damage::{DamageBuffer, DamageInstance},
    game_kinds::CurrentGameKind,
    game_object_spawning::SpawnGameObject,
    stats::components::*,
};
use avian2d::prelude::*;
use bevy::{ecs::query::QueryFilter, prelude::*};
use rand::Rng;

#[derive(Component)]
pub struct WeaponBouncingDice;

#[derive(Component, Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BouncingDiceAttack {
    pub init_pos: Vec2,
    pub c_target: Vec2,
    pub attack_curve: BouncingDiceAttackCurve,
    pub rem_bounces: u8,
    pub time_to_bounce: Timer,
}

/// A function of the form
/// -1.0 * mult_constant (t - time_offset)^2 + curve_height_offset
/// we have to solve for x_offset and the
/// mult constant (which are related) based on the values of the max height offset that
/// we want, and the destination
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BouncingDiceAttackCurve;

impl Curve<f32> for BouncingDiceAttackCurve {
    fn domain(&self) -> Interval {
        Interval::UNIT
    }

    fn sample(&self, t: f32) -> Option<f32> {
        match self.domain().contains(t) {
            true => Some(-4.0 * (t - 0.5).powi(2) + 1.0),
            false => None,
        }
    }

    fn sample_unchecked(&self, t: f32) -> f32 {
        (-4.0 * (t - 0.5).powi(2) + 1.0)
    }
}

pub fn on_activate<QF: QueryFilter>(
    t: On<ActivateWeapon>,
    mut commands: Commands,
    q_weapon: Query<
        (
            &ChildOf,
            &Damage,
            &EffectSize,
            &ProjectileBounces,
            &ProjectileSpeed,
        ),
        (With<WeaponBouncingDice>, QF),
    >,
    q_parent: Query<(&Position, &CharacterFacing)>,
) {
    if let Ok((parent, damage, eff_size, bounces, speed)) = q_weapon.get(t.entity) {
        let (position, facing) = q_parent.get(parent.0).expect("Parent of weapon not found!");
        let facing_vector = facing.c_dir.to_vec();
        let mut rng = rand::rng();
        let angle_offset = rng.random_range(-(TAU / 36.0)..(TAU / 36.0));
        let base_angle = facing_vector.to_angle();
        let new_vec = Vec2::from_angle((base_angle + angle_offset));
        let pos_to_target = position.0 + (speed.0 * new_vec);
        let curve_top = match facing.c_dir {
            FacingDirection::Left | FacingDirection::Right => {
                5.0 * (pos_to_target.y - position.y).abs()
            }
            _ => 1.01 * (pos_to_target.y - position.y),
        };

        commands.queue(SpawnGameObject::new(
            MultiPlayerComponentOptions::PREDICTED,
            (
                *position,
                BouncingDiceAttack {
                    init_pos: position.0,
                    c_target: pos_to_target,
                    attack_curve: BouncingDiceAttackCurve,
                    rem_bounces: bounces.0 as u8,
                    time_to_bounce: Timer::from_seconds(1.0, TimerMode::Once),
                },
                Name::from("Bouncing Dice Attack"),
                *eff_size,
                *damage,
                *bounces,
                *speed,
            ),
        ));
    }
}

pub fn on_deactivate<QF: QueryFilter>(
    trigger: On<DeactivateWeapon>,
    mut commands: Commands,
    q_weapon: Query<(Entity, &CooldownRate), (With<WeaponBouncingDice>, QF)>,
) {
    if let Ok((ent, cdr)) = q_weapon.get(trigger.entity) {
        commands.entity(ent).insert(Cooldown::new(cdr.0));
    }
}

pub fn bouncing_dice_attack<QF: QueryFilter>(
    time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut q_dice: Query<
        (
            Entity,
            &mut Position,
            &mut BouncingDiceAttack,
            &Damage,
            &ProjectileSpeed,
            &EffectSize,
        ),
        QF,
    >,
    mut q_enemies: Query<(Entity, &mut DamageBuffer, &Position), Without<BouncingDiceAttack>>,
) {
    for (ent, mut pos, mut attack, dam, p_speed, eff_size) in &mut q_dice {
        attack.time_to_bounce.tick(time.delta());
        let pct = attack.time_to_bounce.fraction();
        /*
        let curve = QuadraticInOutCurve;
        let pct = curve.sample_unchecked(timer_pct);
        */
        //let pct = attack.attack_curve.sample_unchecked(timer_pct);
        // Lin interp x
        pos.x = pct * attack.c_target.x + (1.0 - pct) * attack.init_pos.x;
        pos.y = pct * attack.c_target.y + (1.0 - pct) * attack.init_pos.y;

        if attack.time_to_bounce.is_finished() {
            let mut entities_to_damage = Vec::new();
            // Damage enemies in an area
            for (ent, _buff, e_pos) in &q_enemies {
                if pos.0.distance(e_pos.0) <= eff_size.0 {
                    entities_to_damage.push(ent.clone())
                } else {
                }
            }

            for e_ent in entities_to_damage {
                let (_, mut buff, _) = q_enemies.get_mut(e_ent).unwrap();
                buff.push(DamageInstance {
                    damage_source: ent,
                    amount: dam.0,
                });
            }
            attack.rem_bounces -= 1;
            if attack.rem_bounces == 0 {
                commands.entity(ent).despawn()
            } else {
                let dir_vec = (attack.c_target - attack.init_pos).normalize_or_zero();
                let mut rng = rand::rng();
                let angle_offset = rng.random_range(-(TAU / 36.0)..(TAU / 36.0));
                let base_angle = dir_vec.to_angle();
                let new_vec = Vec2::from_angle((base_angle + angle_offset));
                let next_pos = attack.c_target + (new_vec * p_speed.0);
                attack.init_pos = attack.c_target;
                attack.c_target = next_pos;
                attack.time_to_bounce.reset()
            }
        }
    }
}
