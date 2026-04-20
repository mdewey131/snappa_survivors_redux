use avian2d::prelude::*;
use bevy::{prelude::*, time::common_conditions::on_timer};
use lightyear::prelude::*;
use rand::Rng;
use std::time::Duration;

use crate::{
    render::enemies::rendering_on_enemy_add,
    shared::{
        colliders::CommonColliderBundle,
        combat::CombatSystemSet,
        enemies::{spawner::*, *},
        game_kinds::{DefaultServerFilter, is_single_player},
        states::{AppState, InGameState},
    },
};

pub struct ServerEnemyRenderPlugin;
impl Plugin for ServerEnemyRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, rendering_on_enemy_add::<With<Replicate>>);
    }
}

pub struct DedicatedServerEnemyPlugin;
impl Plugin for DedicatedServerEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LoadingLevel),
            spawn_enemy_spawn_manager.run_if(not(is_single_player)),
        )
        .add_systems(
            FixedUpdate,
            (
                update_enemy_spawn_manager.run_if(resource_exists::<EnemySpawnManager>),
                update_enemy_spawner,
                enemy_state_machine::<With<Replicate>, With<Replicate>>,
            )
                .run_if(in_state(InGameState::InGame))
                .in_set(CombatSystemSet::Combat),
        )
        .add_observer(add_non_replicated_enemy_components::<DefaultServerFilter>)
        .add_observer(on_enemy_death);
    }
}
