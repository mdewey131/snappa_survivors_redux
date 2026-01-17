use crate::shared::{game_kinds::DefaultServerFilter, projectiles::*, states::*};
use avian2d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::Replicate;

pub struct DedicatedServerProjectilePlugin;

impl Plugin for DedicatedServerProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (projectile_movement::<DefaultServerFilter>,).run_if(in_state(InGameState::InGame)),
        )
        .add_observer(add_projectile_components::<DefaultServerFilter>);
    }
}
