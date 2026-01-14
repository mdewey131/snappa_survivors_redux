use bevy::prelude::*;
use lightyear::prelude::Replicate;

use crate::{
    render::player::rendering_on_player_add,
    shared::{
        colliders::CommonColliderBundle,
        combat::CombatSystemSet,
        players::{Player, player_movement},
    },
};

pub struct ServerPlayerPlugin;

impl Plugin for ServerPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (player_movement::<With<Replicate>>).in_set(CombatSystemSet::Combat),
        )
        .add_observer(handle_player_spawn);
    }
}

pub struct ServerPlayerRenderPlugin;
impl Plugin for ServerPlayerRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, rendering_on_player_add::<With<Replicate>>);
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
            .insert(CommonColliderBundle::player(false));
    }
}
