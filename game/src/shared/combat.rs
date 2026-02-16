use avian2d::math::PI;
use bevy::prelude::*;
use bevy_enhanced_input::{
    action::Action,
    prelude::{ActionValue, Actions},
};

use crate::shared::{inputs::Movement, players::Player};
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

#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect)]
pub enum FacingDirection {
    #[default]
    Down,
    Right,
    Up,
    Left,
}

impl FacingDirection {
    /// Returns an index that can be used
    /// in `AnimationFacing.tex_rows`
    pub fn to_index(&self) -> usize {
        match self {
            FacingDirection::Down => 0,
            FacingDirection::Right => 1,
            FacingDirection::Up => 2,
            FacingDirection::Left => 3,
        }
    }
    pub fn to_vec(&self) -> Vec2 {
        match self {
            FacingDirection::Down => Vec2::NEG_Y,
            FacingDirection::Left => Vec2::NEG_X,
            FacingDirection::Up => Vec2::Y,
            FacingDirection::Right => Vec2::X,
        }
    }
    pub fn from_vec(v: &Vec2) -> Self {
        if v.x > 0.0 {
            if v.y > 0.5 {
                FacingDirection::Up
            } else if v.y < -0.5 {
                FacingDirection::Down
            } else {
                FacingDirection::Right
            }
        } else if v.x < 0.0 {
            if v.y > 0.5 {
                FacingDirection::Up
            } else if v.y < -0.5 {
                FacingDirection::Down
            } else {
                FacingDirection::Left
            }
        } else {
            FacingDirection::Down
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
pub struct CharacterFacing {
    pub c_dir: FacingDirection,
}

impl CharacterFacing {
    pub fn next_direction(&self, direction_vec: Vec2) -> FacingDirection {
        let prev_dir = self.c_dir.to_vec();
        let new_dir = direction_vec.normalize_or_zero();
        let prev_angle = prev_dir.to_angle();
        let new_angle = new_dir.to_angle();
        if (new_angle - prev_angle).abs() > (PI / 8.0) {
            FacingDirection::from_vec(&new_dir)
        } else {
            self.c_dir
        }
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
