use crate::{
    render::enemies::rendering_on_enemy_add,
    shared::{
        colliders::CommonColliderBundle,
        combat::CombatSystemSet,
        enemies::{spawner::*, *},
        game_kinds::{DefaultClientFilter, SinglePlayer, is_single_player},
        states::InGameState,
    },
};
use bevy::prelude::*;
use lightyear::prelude::*;

pub struct ClientEnemyPlugin;

impl Plugin for ClientEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InGameState::InGame),
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
                .in_set(CombatSystemSet::Combat),
        )
        .add_observer(add_non_replicated_enemy_components::<DefaultClientFilter>);
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
