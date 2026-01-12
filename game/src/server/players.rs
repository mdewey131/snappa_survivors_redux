use bevy::prelude::*;
use lightyear::prelude::Replicate;

use crate::shared::{
    colliders::CommonColliderBundle,
    combat::CombatSystemSet,
    players::{Player, player_movement},
};

pub struct ServerPlayerPlugin;

impl Plugin for ServerPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (player_movement::<Replicate>).in_set(CombatSystemSet::Combat),
        )
        .add_observer(handle_player_spawn);
    }
}

fn handle_player_spawn(
    trigger: On<Add, Player>,
    mut commands: Commands,
    q_player: Query<&Player, With<Replicate>>,
) {
    if let Ok(p) = q_player.get(trigger.entity) {
        commands
            .entity(trigger.entity)
            .insert(CommonColliderBundle::from(*p));
    }
}
