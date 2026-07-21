use bevy::{ecs::query::QueryFilter, prelude::*};

use crate::shared::{combat::CombatEntity, states::InGameState};

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct DespawnTimer(pub Timer);

impl DespawnTimer {
    pub fn new(time: f32) -> Self {
        Self(Timer::from_seconds(time, TimerMode::Once))
    }
}

pub struct DespawnTimerPlugin;

impl Plugin for DespawnTimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update)
            .add_systems(
                OnExit(InGameState::InGame),
                pause_timers::<With<CombatEntity>>,
            )
            .add_systems(
                OnEnter(InGameState::InGame),
                unpause_timers::<With<CombatEntity>>,
            );
    }
}

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut q_timer: Query<(Entity, &mut DespawnTimer)>,
) {
    for (ent, mut timer) in &mut q_timer {
        timer.tick(time.delta());
        if timer.just_finished() {
            commands.entity(ent).despawn()
        }
    }
}

fn pause_timers<QF: QueryFilter>(mut q_timer: Query<&mut DespawnTimer, QF>) {
    info!("Pausing!");
    for mut timer in &mut q_timer {
        info!("Found something with a despawn timer");
        timer.pause()
    }
}

fn unpause_timers<QF: QueryFilter>(mut q_timer: Query<&mut DespawnTimer, QF>) {
    for mut timer in &mut q_timer {
        timer.unpause();
    }
}
