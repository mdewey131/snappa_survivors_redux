use std::f32::consts::TAU;

use super::*;
use crate::shared::{
    combat::CharacterFacing, damage::DamageBuffer, game_kinds::CurrentGameKind,
    game_object_spawning::SpawnGameObject, stats::components::*,
};
use avian2d::prelude::*;
use bevy::{ecs::query::QueryFilter, prelude::*};
use rand::Rng;

#[derive(Component)]
pub struct WeaponBouncingDice;

#[derive(Component, Reflect, Debug, Clone)]
pub struct BouncingDiceAttack {
    init_pos: Vec2,
    c_target: Vec2,
    attack_curve: BouncingDiceAttackCurve,
    rem_bounces: u8,
    time_to_bounce: Timer,
}

/// A function of the form
/// -1.0 * mult_constant (t - x_offset)^2 + max_height_offset
/// we have to solve for x_offset and the
/// mult constant (which are related) based on the values of the max height offset that
/// we want, and the destination
#[derive(Reflect, Debug, Clone, Copy)]
pub struct BouncingDiceAttackCurve {
    max_height_offset: f32,
    mult_constant: f32,
    x_offset: f32,
}
impl BouncingDiceAttackCurve {
    fn new(max_height_offset: f32, destination_height_offset: f32) -> Self {
        // Solve for mult constant. Much math. Trust me bro?
        let mult_constant = (max_height_offset.sqrt()
            + (max_height_offset + destination_height_offset).sqrt())
        .powi(2);
        let x_offset = (max_height_offset / mult_constant).sqrt();
        Self {
            max_height_offset,
            mult_constant,
            x_offset,
        }
    }
}

impl Curve<f32> for BouncingDiceAttackCurve {
    fn domain(&self) -> Interval {
        Interval::UNIT
    }

    fn sample(&self, t: f32) -> Option<f32> {
        match self.domain().contains(t) {
            true => Some(
                -1.0 * self.mult_constant * (t - self.x_offset).powi(2) + self.max_height_offset,
            ),
            false => None,
        }
    }

    fn sample_unchecked(&self, t: f32) -> f32 {
        -1.0 * self.mult_constant * (t - self.x_offset).powi(2) + self.max_height_offset
    }
}

pub fn on_activate<QF: QueryFilter>(
    t: On<ActivateWeapon>,
    mut commands: Commands,
    q_weapon: Query<
        (&ChildOf, &Damage, &EffectSize, &ProjectileBounces),
        (With<WeaponBouncingDice>, QF),
    >,
    q_parent: Query<(&Position, &CharacterFacing)>,
) {
    if let Ok((parent, damage, eff_size, bounces)) = q_weapon.get(t.entity) {
        let (position, facing) = q_parent.get(parent.0).expect("Parent of weapon not found!");
        let facing_vector = facing.c_dir.to_vec();
        let mut rng = rand::rng();
        let angle_offset = rng.random_range(-(TAU / 36.0)..(TAU / 36.0));
        let base_angle = facing_vector.to_angle();
        let new_vec = Vec2::from_angle((base_angle + angle_offset));
        let pos_to_target = position.0 + (10.0 * new_vec);
        let curve = BouncingDiceAttackCurve::new(10.0, (pos_to_target.y - position.y));

        commands.queue(SpawnGameObject::new(
            MultiPlayerComponentOptions::PREDICTED,
            (
                position.clone(),
                BouncingDiceAttack {
                    init_pos: position.0,
                    c_target: pos_to_target,
                    attack_curve: curve,
                    rem_bounces: bounces.0 as u8,
                    time_to_bounce: Timer::from_seconds(0.2, TimerMode::Once),
                },
                Name::from("Bouncing Dice Attack"),
                *eff_size,
                *damage,
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
    mut q_dice: Query<(&mut Position, &mut BouncingDiceAttack, &Damage), QF>,
    mut q_enemies: Query<(&mut DamageBuffer, &Position), Without<BouncingDiceAttack>>,
) {
    for (mut pos, mut attack, dam) in &mut q_dice {
        attack.time_to_bounce.tick(time.delta());
        let pct = attack.time_to_bounce.fraction();
        pos.x = (attack.c_target.x - attack.init_pos.x) * pct;
        pos.y = attack.attack_curve.sample_unchecked(pct)
    }
}
