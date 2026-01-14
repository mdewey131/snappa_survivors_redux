use crate::{
    render::enemies::rendering_on_enemy_add,
    shared::{
        colliders::CommonColliderBundle, combat::CombatSystemSet, enemies::*,
        game_kinds::SinglePlayer,
    },
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
        .add_systems(FixedPreUpdate, (add_missing_enemy_components));
    }
}

pub struct ClientEnemyRenderPlugin;
impl Plugin for ClientEnemyRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, rendering_on_enemy_add::<With<Replicate>>);
    }
}

/// These add the components, for a spawn timer and colliders
/// this cannot be run off of a trigger, because SinglePlayer is also added off of a trigger
fn add_missing_enemy_components(
    mut commands: Commands,
    q_to_attach: Query<(Entity, &Enemy), (Added<Enemy>, Or<(With<Predicted>, With<SinglePlayer>)>)>,
) {
    for (ent, en) in &q_to_attach {
        match en.state {
            EnemyState::Spawning => {
                commands.entity(ent).insert((EnemySpawnTimer::default()));
            }
            _ => {}
        }
        commands
            .entity(ent)
            .insert(CommonColliderBundle::enemy(true));
    }
}
