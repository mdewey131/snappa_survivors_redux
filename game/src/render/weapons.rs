use crate::{render::RenderYtoZ, shared::weapons::*};
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
            .insert((Sprite::from(img), RenderYtoZ));
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
                .insert((Sprite::from(img), RenderYtoZ, *t_pos));
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
