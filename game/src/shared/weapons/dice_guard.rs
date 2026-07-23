use crate::{
    shared::{
        colliders::*,
        combat::Cooldown,
        game_kinds::{CurrentGameKind, MultiPlayerComponentOptions},
        game_object_spawning::spawn_game_object,
        projectiles::Projectile,
        stats::components::*,
        weapons::DeactivateWeapon,
    },
    utils::{CreatedBy, CreatorOf},
};

use super::ActivateWeapon;
use crate::shared::{players::Player, projectiles::*};
use avian2d::prelude::*;
use bevy::{ecs::query::QueryFilter, prelude::*};
use serde::{Deserialize, Serialize};

/// Marker component for a weapon
#[derive(Component, Default)]
pub struct DiceGuard {
    pub projectiles: Option<Vec<Entity>>,
}

#[derive(Component, Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct DiceGuardProjectile;

pub fn dice_guard_activate<QF: QueryFilter>(
    trigger: On<ActivateWeapon>,
    mut commands: Commands,
    game_kind: Res<CurrentGameKind>,
    mut q_dice_guards: Query<
        (
            Entity,
            &ChildOf,
            &ProjectileCount,
            &EffectSize,
            &ProjectileSpeed,
            &Damage,
            &mut DiceGuard,
        ),
        (QF),
    >,
    q_parent: Query<&Position, With<Player>>,
) {
    if let Ok((dg_ent, parent, p_count, size, speed, dam, mut dg)) =
        q_dice_guards.get_mut(trigger.entity)
    {
        let mut projectiles = vec![];
        info!("Dice guard activated!");
        let par_pos = q_parent.get(parent.parent()).unwrap();
        /*
        let spawn_positions = SpawnStrategy::Circle {
            center: par_pos.0,
            num: p_count as usize,
            radius: size,
        };
         */
        let iters = p_count.0.floor() as usize;
        for i in 0..iters {
            // Shorhand for now
            let r = size.0 * 4.0;
            //spawn_positions.positions_2d().into_iter().enumerate() {
            let angle = std::f32::consts::TAU * (i as f32 / p_count.0);
            let proj = Projectile {
                movement: ProjectileMovement::Orbital {
                    around: parent.parent(),
                    speed: speed.0,
                    c_angle: angle,
                    radius: r,
                },
            };
            let pos = par_pos.0 + Vec2::from_angle(angle) * r;
            trace!("Found angle to be {angle}, position is {:?}", pos);
            let ent = spawn_game_object(
                &mut commands,
                game_kind.0.unwrap(),
                None::<()>,
                MultiPlayerComponentOptions::from(proj),
                (
                    proj,
                    DiceGuardProjectile,
                    Position(pos),
                    CreatedBy(parent.0),
                    *dam,
                    *size,
                    AppliesCollisionEffect::new(
                        [ColliderTypes::Enemy].into(),
                        ApplyDamage::default(),
                    ),
                ),
            );
            projectiles.push(ent);
        }
        dg.projectiles = Some(projectiles);
    }
}

pub fn dice_guard_deactivate<QF: QueryFilter>(
    trigger: On<DeactivateWeapon>,
    mut commands: Commands,
    mut q_dice_guards: Query<(Entity, &CooldownRate, &mut DiceGuard), (With<DiceGuard>, QF)>,
) {
    if let Ok((ent, cdr, mut dg)) = q_dice_guards.get_mut(trigger.entity) {
        let projectiles = dg.projectiles.take();
        if let Some(ps) = projectiles {
            for proj in ps {
                commands.entity(proj).despawn();
            }
        }
        commands.entity(ent).insert(Cooldown::new(cdr.0));
    }
}
