use crate::{
    render::RenderYtoZ,
    shared::{projectiles::*, weapons::*},
};
use bevy::{ecs::query::QueryFilter, prelude::*};

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
