use crate::{
    render::RenderYtoZ,
    shared::{stats::components::ProjectileBounces, weapons::*},
};
use bevy::{ecs::query::QueryFilter, prelude::*};
const THROW_HANDS_SPRITE_Y_OFFSET: f32 = -0.01;

pub fn add_dice_guard_rendering_components<QF: QueryFilter>(
    t: On<Add, DiceGuardProjectile>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    q_projectile: Query<(), QF>,
) {
    if let Ok(()) = q_projectile.get(t.entity) {
        let img: Handle<Image> = assets.load("weapons/dice_guard/projectile.png");
        commands
            .entity(t.entity)
            .insert((Sprite::from(img), RenderYtoZ::default()));
    }
}

pub fn add_throwing_hands_attack_sprite<QF: QueryFilter>(
    t: On<Add, ThrowHandsAttack>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    q_attack: Query<&ThrowHandsAttack, QF>,
    q_target: Query<&Transform>,
) {
    if let Ok(atk) = q_attack.get(t.entity) {
        let img: Handle<Image> = assets.load("weapons/throw_hands/attack.png");
        if let Ok(t_pos) = q_target.get(atk.target) {
            commands
                .entity(t.entity)
                .insert((Sprite::from(img), RenderYtoZ::new(0.05), *t_pos));
        }
    }
}

pub fn throwing_hands_sprite_follow<QF: QueryFilter>(
    mut q_sprite: Query<(&mut Transform, &ThrowHandsAttack), /*(With<ThrowHandsAttack>, */ QF>,
    q_target: Query<&Transform, Without<ThrowHandsAttack>>,
) {
    for (mut pos, atk) in &mut q_sprite {
        if let Ok(t_pos) = q_target.get(atk.target) {
            pos.translation = t_pos.translation + (THROW_HANDS_SPRITE_Y_OFFSET * Vec3::Y);
        }
    }
}

/// Marker component for the sprite that gets drawn at the location where the dice is bouncing to
/// Ideally, this spawns as a child element becuase we want to despawn this whenever the attack is despawned
#[derive(Component)]
#[relationship(relationship_target = HasBouncingDiceTarget)]
pub struct BouncingDiceTarget {
    target_of: Entity,
}

#[derive(Component)]
#[relationship_target(relationship = BouncingDiceTarget, linked_spawn)]
pub struct HasBouncingDiceTarget(Entity);

pub fn add_bouncing_dice_rendering_components<QF: QueryFilter>(
    t: On<Add, BouncingDiceAttack>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    q_projectile: Query<&BouncingDiceAttack, QF>,
) {
    if let Ok(attack) = q_projectile.get(t.entity) {
        let dice_handle: Handle<Image> = assets.load("weapons/bouncing_dice/projectile.png");
        let target_handle: Handle<Image> = assets.load("weapons/bouncing_dice/target.png");

        let dice_entity = commands
            .entity(t.entity)
            .insert(Sprite::from(dice_handle) )
            .id();

        let _target_ent = commands
            .spawn((
                Sprite::from(target_handle),
                Transform::from_translation(attack.c_target.extend(attack.c_target.y)),
                RenderYtoZ::new(10.0),
                BouncingDiceTarget {
                    target_of: dice_entity,
                },
            ))
            .id();
    }
}

pub fn update_bouncing_dice_render_components<QF: QueryFilter>(
    mut q_projectile: Query<
        (
            &mut Transform,
            &BouncingDiceAttack,
            &ProjectileBounces,
            &HasBouncingDiceTarget,
        ),
        QF ,
    >,
    mut q_target: Query<&mut Transform, (With<BouncingDiceTarget>, Without<BouncingDiceAttack>)>,
) {
    for (mut transform, attack, bounces, target_ent) in &mut q_projectile {
        // scales according to parabola that goes up to 1.0
        let pct = attack.time_to_bounce.fraction();
        if (bounces.0 as u8) == attack.rem_bounces && pct < 0.5 {
            transform.scale = Vec2::splat((-8.0 * (pct - 0.5).powi(2)) + 2.0).extend(0.0);
        }
        transform.scale = Vec3::ONE + Vec2::splat((-4.0 * (pct - 0.5).powi(2)) + 1.0).extend(0.0);
        if let Ok(mut t_pos) = q_target.get_mut(target_ent.0) {
            if t_pos.translation.xy() == attack.c_target {
            } else {
                t_pos.translation.x = attack.c_target.x;
                t_pos.translation.y = attack.c_target.y;
            }
        }
    }
}
