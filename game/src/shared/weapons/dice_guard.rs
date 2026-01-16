use crate::shared::projectiles::Projectile;

use super::ActivateWeapon;
use avian2d::prelude::*;
use bevy::{ecs::query::QueryFilter, prelude::*};
use crate::shared::{players::Player, projectiles::*};

/// Marker component for a weapon
#[derive(Component)]
pub struct DiceGuard;

#[derive(Component)]
pub struct DiceGuardProjectile;

pub fn dice_guard_activate<QF: QueryFilter>(
    trigger: On<ActivateWeapon>,
    mut commands: Commands,
    q_dice_guards: Query<(Entity /*,  &StatsList*/, &ChildOf), (With<DiceGuard>, QF)>,
    q_parent: Query<&Position, With<Player>>,
) {
    if let Ok((dg_ent /*, stats */, parent)) = q_dice_guards.get(trigger.entity) {
        info!("Dice guard activated!");
        let par_pos = q_parent.get(parent.parent()).unwrap();
        let p_count = 3.0; //**stats.get_current(StatKind::ProjectileCount).unwrap();
        let size = 5.0; //**stats.get_current(StatKind::EffectSize).unwrap();
        let speed = 50.0; //**stats.get_current(StatKind::ProjectileSpeed).unwrap();
        /*
        let spawn_positions = SpawnStrategy::Circle {
            center: par_pos.0,
            num: p_count as usize,
            radius: size,
        };
         */
        for (i, pos) in vec![Vec2::X, Vec2::Y, Vec2::NEG_X, Vec2::NEG_Y].iter().enumerate(){ //spawn_positions.positions_2d().into_iter().enumerate() {
            let angle = std::f32::consts::TAU * (i as f32 / p_count);
            trace!("Found angle to be {angle}, position is {:?}", pos);
            commands.spawn((
                DiceGuardProjectile,
                Projectile {
                    movement: ProjectileMovement::Linear(*pos)
                },
                Position(*pos),
            ));
        }
    }
}

/*
fn dice_guard_deactivate(
    trigger: On<DeactivateWeapon>,
    mut commands: Commands,
    q_dice_guards: Query<(Entity, &CreatorOf, &StatsList), With<DiceGuard>>,
) {
    if let Ok((ent, created, stats)) = q_dice_guards.get(trigger.entity) {
        for proj in created.iter() {
            commands.entity(proj).despawn();
        }
        let cd = **stats.get_current(StatKind::CooldownRate).unwrap();
        commands.entity(ent).insert(Cooldown::new(cd));
    }
}
*/
