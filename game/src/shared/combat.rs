use bevy::prelude::*;
#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Clone, Copy, Default)]
pub enum CombatSystemSet {
    /// Used for anything that should make itself known to combat beforehand (e.g. spawning bullets, leveling))
    #[default]
    PreCombat,
    Combat,
    /// Apply what you need immediately following the combat step, but still in `FixedUpdate`.
    PostCombatUpdate,
    /// Runs things like updating collider positions and checking for damage, in `FixedPostUpdate`
    PostPhysicsSet,
    /// Finally resovles the DamageBuffer
    Cleanup,
    Last,
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            FixedUpdate,
            (
                CombatSystemSet::PreCombat,
                CombatSystemSet::Combat,
                CombatSystemSet::PostCombatUpdate,
            )
                .chain(), /*
                          // This does nothing at the moment, per https://github.com/bevyengine/bevy/issues/13064
                          .run_if(in_state(InGameState::InGame)),
                          */
        )
        .configure_sets(
            FixedPostUpdate,
            (
                CombatSystemSet::PostPhysicsSet,
                CombatSystemSet::Cleanup,
                CombatSystemSet::Last,
            )
                .chain(),
        )
        .add_systems(FixedPreUpdate, tick_cooldown);
    }
}

/// To be used anytime something is on cooldown (duh)
#[derive(Component, Clone, Deref, DerefMut)]
pub struct Cooldown(Timer);
impl Cooldown {
    pub fn new(time: f32) -> Self {
        Cooldown(Timer::from_seconds(time, TimerMode::Once))
    }
}

fn tick_cooldown(
    time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut q_cooldown: Query<(Entity, &mut Cooldown)>,
) {
    for (ent, mut cd) in &mut q_cooldown {
        cd.tick(time.delta());
        if cd.just_finished() {
            commands.entity(ent).remove::<Cooldown>();
        }
    }
}
