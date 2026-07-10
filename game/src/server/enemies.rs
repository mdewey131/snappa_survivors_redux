use bevy::prelude::*;
use lightyear::prelude::*;

use crate::{
    render::enemies::rendering_on_enemy_add,
    shared::{
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
                (
                    update_enemy_spawn_manager.run_if(resource_exists::<EnemySpawnManager>),
                    update_enemy_spawner,
                    enemy_state_machine::<With<Replicate>, With<Replicate>>,
                )
                    .run_if(in_state(InGameState::InGame))
                    .in_set(CombatSystemSet::Combat),
                (
                    check_enemy_death::<DefaultServerFilter>,
                    while_enemy_dead::<DefaultServerFilter>,
                )
                    .run_if(in_state(InGameState::InGame))
                    .in_set(CombatSystemSet::Last),
            ),
        )
        .add_observer(add_non_replicated_enemy_components::<DefaultServerFilter>);
    }
}
