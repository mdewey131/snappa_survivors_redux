use crate::shared::{game_kinds::DefaultServerFilter, projectiles::*, states::*};
use avian2d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::Replicate;

pub struct DedicatedServerProjectilePlugin;

impl Plugin for DedicatedServerProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                add_projectile_components::<Added<Replicate>>,
                projectile_movement::<DefaultServerFilter>,
            )
                .run_if(in_state(InGameState::InGame)),
        );
    }
}

/// Just something simple for testing
fn spawn_projectile(mut commands: Commands) {
    commands.spawn((
        Projectile {
            movement: ProjectileMovement::Linear(Vec2::Y),
        },
        Position(Vec2::ZERO),
    ));
}
