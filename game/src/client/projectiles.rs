use crate::shared::{colliders::CommonColliderBundle, game_kinds::*, projectiles::*};
use bevy::prelude::*;
use lightyear::prelude::*;

pub struct ClientProjectilePlugin;

impl Plugin for ClientProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, projectile_movement::<DefaultClientFilter>);
    }
}

fn add_projectile_components(
    mut commands: Commands,
    q_projectile: Query<
        Entity,
        (
            With<Projectile>,
            Or<(Added<Predicted>, Added<SinglePlayer>)>,
        ),
    >,
) {
    for p in &q_projectile {
        commands.entity(p).insert(())
    }
}
