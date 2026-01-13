use crate::shared::{colliders::CommonColliderBundle, combat::CombatSystemSet, enemies::*};
use bevy::prelude::*;
use lightyear::prelude::*;

pub struct ClientEnemyPlugin;

impl Plugin for ClientEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (enemy_state_machine::<With<Predicted>, With<Predicted>>)
                .in_set(CombatSystemSet::Combat),
        )
        .add_observer(on_predicted_enemy_spawn)
        .add_observer(add_enemy_colliders::<With<Predicted>>);
    }
}

fn on_predicted_enemy_spawn(
    trigger: On<Add, Enemy>,
    mut commands: Commands,
    q_to_attach: Query<&Enemy, With<Predicted>>,
) {
    if let Ok(e) = q_to_attach.get(trigger.entity) {
        match e.state {
            EnemyState::Spawning => {
                commands
                    .entity(trigger.entity)
                    .insert((EnemySpawnTimer::default()));
            }
            _ => {}
        }
    }
}
