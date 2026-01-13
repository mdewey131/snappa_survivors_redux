use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use std::time::Duration;

#[derive(Component, Reflect)]
pub struct AnimationConfig {
    pub first_sprite_index: usize,
    pub last_sprite_index: usize,
    pub fps: u8,
    pub frame_timer: Timer,
}

impl AnimationConfig {
    pub fn new(first: usize, last: usize, fps: u8) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps),
        }
    }

    fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }
}

/// Handles the managment of a character with a facing,
/// such as the `Player` or `Enemy`
///
/// This holds a reference to the last direction that the
/// unit was facing based on movement, so that it can detect when
/// to change the facing based on "this" frame
#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct AnimationFacing {
    pub last_frame_dir: Vec2,
    pub c_dir: FacingDirection,
    /// Records which texture row has each facing,
    /// defaults in the order Down, Right, Up, Left [0, 1, 2, 3]
    pub tex_rows: [usize; 4],
    // Records how wide the spritesheet is.
    // this is helpful so that you don't have to store the
    // total size of the sprite
    pub tex_width: u32,
}

impl AnimationFacing {
    /// Returns the old facing direction
    fn derive_next_direction(&mut self, c_velo: Vec2) -> FacingDirection {
        let old_dir = self.c_dir;
        // Normalize current velocity for a direction
        let dir = c_velo.normalize_or_zero();
        if dir != Vec2::ZERO {
            let facing = if dir.x.abs() == 1.0 || dir.y.abs() == 1.0 {
                // We're for sure setting a direction
                if dir.x > 0.0 {
                    FacingDirection::Right
                } else if dir.x < 0.0 {
                    FacingDirection::Left
                } else if dir.y > 0.0 {
                    FacingDirection::Up
                } else {
                    FacingDirection::Down
                }
            } else {
                // Corner cases
                // Keeping it somewhat simple for now
                match self.c_dir {
                    FacingDirection::Up => {
                        if dir.y < 0.0 {
                            FacingDirection::Down
                        } else {
                            FacingDirection::Up
                        }
                    }
                    FacingDirection::Down => {
                        if dir.y > 0.0 {
                            FacingDirection::Up
                        } else {
                            FacingDirection::Down
                        }
                    }
                    FacingDirection::Right => {
                        if dir.x < 0.0 {
                            FacingDirection::Left
                        } else {
                            FacingDirection::Right
                        }
                    }
                    FacingDirection::Left => {
                        if dir.x > 0.0 {
                            FacingDirection::Right
                        } else {
                            FacingDirection::Left
                        }
                    }
                }
            };
            if facing != self.c_dir {
                self.c_dir = facing;
            }
        }
        self.last_frame_dir = dir;
        return old_dir;
    }
}

impl Default for AnimationFacing {
    fn default() -> Self {
        Self {
            tex_rows: [0, 1, 2, 3],
            last_frame_dir: Vec2::ZERO,
            c_dir: FacingDirection::Down,
            tex_width: 1,
        }
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
    fn to_index(&self) -> usize {
        match self {
            FacingDirection::Down => 0,
            FacingDirection::Right => 1,
            FacingDirection::Up => 2,
            FacingDirection::Left => 3,
        }
    }
}

pub fn animate<C: Component>(
    time: Res<Time<Virtual>>,
    mut q_anim: Query<(&mut AnimationConfig, &mut Sprite), With<C>>,
) {
    for (mut config, mut sprite) in &mut q_anim {
        config.frame_timer.tick(time.delta());

        if config.frame_timer.just_finished() {
            trace!("Finished Animation Timer");
            let atlas = if let Some(ref mut a) = sprite.texture_atlas {
                a
            } else {
                warn!("Animating a Sprite without a texture atlas!");
                return;
            };
            trace!("Sprite Atlas: {}", atlas.index);
            if atlas.index == config.last_sprite_index {
                atlas.index = config.first_sprite_index;
            } else {
                atlas.index += 1;
            }
            config.frame_timer.reset();
        }
    }
}

pub fn update_facing_direction<C: Component>(
    mut q_animation: Query<
        (
            &mut AnimationFacing,
            &mut AnimationConfig,
            &mut Sprite,
            &LinearVelocity,
        ),
        With<C>,
    >,
) {
    for (mut facing, mut config, mut sprite, velo) in &mut q_animation {
        facing.derive_next_direction(velo.0);
        // TODO: Move this elsewhere
        if velo.0 == Vec2::ZERO {
            config.frame_timer.pause()
        } else if config.frame_timer.is_paused() {
            config.frame_timer.unpause()
        }
        if let Some(ref mut tex) = sprite.texture_atlas {
            let c_idx = tex.index;
            let diff = c_idx - config.first_sprite_index;
            let min = (facing.tex_width as usize * facing.tex_rows[facing.c_dir.to_index()]);
            let max = min + (facing.tex_width as usize) - 1;
            let new = min + diff;
            tex.index = new;
            config.first_sprite_index = min;
            config.last_sprite_index = max;

            //let new_base = facing.c_dir.to_index() *
            //facing.c_dir.to_index()
        }
    }
}
