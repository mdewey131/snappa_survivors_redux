use crate::{
    render::enemies::rendering_on_enemy_add,
    shared::{
        combat::CombatSystemSet,
        enemies::{spawner::*, *},
        game_kinds::{DefaultClientFilter, SinglePlayer, is_single_player},
        states::{AppState, InGameState},
    },
};
use bevy::prelude::*;
use lightyear::prelude::*;

pub struct ClientEnemyPlugin;

impl Plugin for ClientEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            spawn_enemy_spawn_manager.run_if(is_single_player),
        )
        .add_systems(
            FixedUpdate,
            (
                update_enemy_spawn_manager.run_if(resource_exists::<EnemySpawnManager>),
                enemy_state_machine::<
                    Or<(With<Predicted>, With<SinglePlayer>)>,
                    Or<(With<Predicted>, With<SinglePlayer>)>,
                >,
            )
                .in_set(CombatSystemSet::Combat)
                .run_if(in_state(InGameState::InGame)),
        )
        .add_observer(add_non_replicated_enemy_components::<DefaultClientFilter>)
        .add_observer(on_enemy_death);
    }
}

pub struct ClientEnemyRenderPlugin;
impl Plugin for ClientEnemyRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            rendering_on_enemy_add::<Or<(With<SinglePlayer>, With<Predicted>)>>,
        );
    }
}
