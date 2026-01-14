use crate::shared::{
    colliders::CommonColliderBundle, combat::CombatSystemSet, enemies::*, game_kinds::SinglePlayer,
};
use bevy::prelude::*;
use lightyear::prelude::*;

pub struct ClientEnemyPlugin;

impl Plugin for ClientEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (enemy_state_machine::<
                Or<(With<Predicted>, With<SinglePlayer>)>,
                Or<(With<Predicted>, With<SinglePlayer>)>,
            >)
                .in_set(CombatSystemSet::Combat),
        )
        .add_observer(on_predicted_enemy_spawn);
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
        commands
            .entity(trigger.entity)
            .insert(CommonColliderBundle::enemy(true));
    }
}
