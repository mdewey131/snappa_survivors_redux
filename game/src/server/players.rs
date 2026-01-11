use bevy::prelude::*;
use lightyear::prelude::Replicate;

use crate::shared::{
    colliders::CommonColliderBundle,
    players::{Player, player_movement},
};

pub struct ServerPlayerPlugin;

impl Plugin for ServerPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(player_movement::<Replicate>)
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
