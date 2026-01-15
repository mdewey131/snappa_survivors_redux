use crate::shared::{game_kinds::DefaultServerFilter, projectiles::*, states::*};
use bevy::prelude::*;

pub struct DedicatedServerProjectilePlugin;

impl Plugin for DedicatedServerProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            projectile_movement::<DefaultServerFilter>.run_if(in_state(InGameState::InGame)),
        );
    }
}
