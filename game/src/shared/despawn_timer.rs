use bevy::prelude::*;

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct DespawnTimer(Timer);

impl DespawnTimer {
    pub fn new(time: f32) -> Self {
        Self(Timer::from_seconds(time, TimerMode::Once))
    }
}

pub struct DespawnTimerPlugin;

impl Plugin for DespawnTimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
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
