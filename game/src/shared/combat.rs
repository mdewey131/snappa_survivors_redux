use avian2d::math::PI;
use bevy::prelude::*;
use lightyear::prelude::*;
use rand::{SeedableRng, rngs::SmallRng};
use serde::{Deserialize, Serialize};

use crate::shared::damage::DeathState;
pub type EntityIncapacitated = With<DeathState>;
pub type CombatEntityActive = Without<DeathState>;

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
    /// Finally resovles the HealthBuffer
    Cleanup,
    Last,
}

#[derive(Resource)]
pub struct CombatManager {
    pub rng: SmallRng,
}

impl Default for CombatManager {
    fn default() -> Self {
        Self {
            rng: SmallRng::from_seed([0 as u8; 32]),
        }
    }
}

/// A simple marker component to help with some special behavior when states transition ,
/// without invalidating the simplicity of other plugins
#[derive(Component, Debug, Serialize, Deserialize, Clone, Default)]
pub struct CombatEntity;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CombatManager>()
            .configure_sets(
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

pub struct CombatProtocolPlugin;

impl Plugin for CombatProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.component::<CharacterFacing>().predict();
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

#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Serialize, Deserialize)]
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

#[derive(Component, Debug, Clone, Copy, Reflect, Default, PartialEq, Serialize, Deserialize)]
pub struct CharacterFacing {
    pub c_dir: FacingDirection,
}

impl CharacterFacing {
    pub fn next_direction(&self, direction_vec: Vec2) -> FacingDirection {
        let prev_dir = self.c_dir.to_vec();
        let new_dir = direction_vec.normalize_or_zero();
        let prev_angle = prev_dir.to_angle();
        let new_angle = new_dir.to_angle();
        trace!("New angle {:?}", new_angle);
        if new_angle == 0.0 && new_dir != Vec2::X {
            self.c_dir
        } else if (new_angle - prev_angle).abs() <= (PI / 8.0) {
            self.c_dir
        } else {
            if new_dir.x >= 0.0 {
                if new_dir.x > 0.5 {
                    FacingDirection::Right
                } else {
                    if new_dir.y > 0.0 {
                        FacingDirection::Up
                    } else {
                        FacingDirection::Down
                    }
                }
            } else {
                if new_dir.x < -0.5 {
                    FacingDirection::Left
                } else {
                    if new_dir.y > 0.0 {
                        FacingDirection::Up
                    } else {
                        FacingDirection::Down
                    }
                }
            }
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
