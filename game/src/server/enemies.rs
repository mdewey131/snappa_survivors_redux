use avian2d::prelude::*;
use bevy::{prelude::*, time::common_conditions::on_timer};
use lightyear::prelude::*;
use rand::Rng;
use std::time::Duration;

use crate::shared::{
    colliders::CommonColliderBundle, combat::CombatSystemSet, enemies::*, states::InGameState,
};

pub struct ServerEnemyPlugin;

impl Plugin for ServerEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                spawn_enemy.run_if(on_timer(Duration::from_secs(20))),
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
        app.add_observer(add_enemy_colliders::<With<Replicate>>);
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
        CommonColliderBundle::from(enemy),
        enemy,
        Position(Vec2::new(pos.0, pos.1)),
        EnemySpawnTimer::default(),
        Replicate::to_clients(NetworkTarget::All),
        PredictionTarget::to_clients(NetworkTarget::All),
    ));
}
