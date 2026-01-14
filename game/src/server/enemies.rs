use avian2d::prelude::*;
use bevy::{prelude::*, time::common_conditions::on_timer};
use lightyear::prelude::*;
use rand::Rng;
use std::time::Duration;

use crate::shared::{
    colliders::CommonColliderBundle, combat::CombatSystemSet, enemies::*,
    game_kinds::is_single_player, states::InGameState,
};

pub struct ServerEnemyPlugin;

impl Plugin for ServerEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                // This use of the check of whether or not the server resource exists
                // is a bit of a hack. In theory, the way that this should work is we
                // should be checking whether or not the game mode is in multiplayer,
                // and if so then the client shouldn't be running this. But, because it's
                // in the client since that gets the server plugin, this causes problems as
                // written without the check
                spawn_enemy.run_if(on_timer(Duration::from_secs(20)).and(is_single_player)),
                enemy_state_machine::<With<Replicate>, With<Replicate>>,
            )
                .run_if(in_state(InGameState::InGame))
                .in_set(CombatSystemSet::Combat),
        );
    }
}

pub struct DedicatedServerEnemyPlugin;
impl Plugin for DedicatedServerEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            spawn_enemy.run_if(on_timer(Duration::from_secs(20)).and(is_single_player)),
        );
    }
}

fn spawn_enemy(mut commands: Commands) {
    let enemy = Enemy {
        kind: EnemyKind::FacelessMan,
        state: EnemyState::Spawning,
    };
    let mut rng = rand::rng();
    let pos = (rng.random_range(-50.0..50.0), rng.random_range(-50.0..50.0));
    commands.spawn((
        CommonColliderBundle::enemy(false),
        enemy,
        Position(Vec2::new(pos.0, pos.1)),
        EnemySpawnTimer::default(),
        Replicate::to_clients(NetworkTarget::All),
        PredictionTarget::to_clients(NetworkTarget::All),
    ));
}
