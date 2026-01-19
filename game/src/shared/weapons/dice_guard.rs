use crate::{
    shared::{
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
#[derive(Component)]
pub struct DiceGuard;

#[derive(Component, Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct DiceGuardProjectile;

pub fn dice_guard_activate<QF: QueryFilter>(
    trigger: On<ActivateWeapon>,
    mut commands: Commands,
    game_kind: Res<CurrentGameKind>,
    q_dice_guards: Query<
        (
            Entity,
            &ChildOf,
            &ProjectileCount,
            &EffectSize,
            &ProjectileSpeed,
            &Damage,
        ),
        (With<DiceGuard>, QF),
    >,
    q_parent: Query<&Position, With<Player>>,
) {
    if let Ok((dg_ent, parent, p_count, size, speed, dam)) = q_dice_guards.get(trigger.entity) {
        info!("Dice guard activated!");
        let par_pos = q_parent.get(parent.parent()).unwrap();
        /*
        let spawn_positions = SpawnStrategy::Circle {
            center: par_pos.0,
            num: p_count as usize,
            radius: size,
        };
         */
        for i in (0..p_count.0) {
            // Shorhand for now
            let r = size.0 * 4.0;
            //spawn_positions.positions_2d().into_iter().enumerate() {
            let angle = std::f32::consts::TAU * (i as f32 / p_count.0 as f32);
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
            spawn_game_object(
                &mut commands,
                game_kind.0.unwrap(),
                MultiPlayerComponentOptions::from(proj),
                (
                    proj,
                    DiceGuardProjectile,
                    Position(pos),
                    CreatedBy(dg_ent),
                    *dam,
                    *size,
                ),
            );
        }
    }
}

pub fn dice_guard_deactivate<QF: QueryFilter>(
    trigger: On<DeactivateWeapon>,
    mut commands: Commands,
    q_dice_guards: Query<(Entity, &CreatorOf, &CooldownRate), (With<DiceGuard>, QF)>,
) {
    if let Ok((ent, created, cdr)) = q_dice_guards.get(trigger.entity) {
        for proj in created.iter() {
            commands.entity(proj).despawn();
        }
        commands.entity(ent).insert(Cooldown::new(cdr.0));
    }
}
