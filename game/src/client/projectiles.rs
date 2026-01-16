use crate::shared::{
    colliders::CommonColliderBundle, combat::CombatSystemSet, game_kinds::*, projectiles::*,
    states::InGameState,
};
use bevy::prelude::*;
use lightyear::prelude::*;

pub struct ClientProjectilePlugin;

impl Plugin for ClientProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                add_projectile_components::<Or<(Added<Predicted>, Added<SinglePlayer>)>>,
                projectile_movement::<DefaultClientFilter>,
            )
                .in_set(CombatSystemSet::Combat)
                .run_if(in_state(InGameState::InGame)),
        );
    }
}
