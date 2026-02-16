use avian2d::{math::PI, prelude::LinearVelocity};
use bevy::prelude::*;
use std::time::Duration;

use crate::shared::combat::{CharacterFacing, FacingDirection};

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
    /// Records which texture row has each facing,
    /// defaults in the order Down, Right, Up, Left [0, 1, 2, 3]
    pub tex_rows: [usize; 4],
    // Records how wide the spritesheet is.
    // this is helpful so that you don't have to store the
    // total size of the sprite
    pub tex_width: u32,
}
impl AnimationFacing {
    pub fn update_facing(
        &mut self,
        facing: FacingDirection,
        config: &mut Mut<AnimationConfig>,
        sprite: &mut Mut<Sprite>,
    ) {
        if let Some(ref mut tex) = sprite.texture_atlas {
            let c_idx = tex.index;
            let diff = c_idx - config.first_sprite_index;
            let min = (self.tex_width as usize * self.tex_rows[facing.to_index()]);
            let max = min + (self.tex_width as usize) - 1;
            let new = min + diff;
            tex.index = new;
            config.first_sprite_index = min;
            config.last_sprite_index = max;
        }
    }
}

impl Default for AnimationFacing {
    fn default() -> Self {
        Self {
            tex_rows: [0, 1, 2, 3],
            tex_width: 1,
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

/// This shows a way to do facing that works off of velocity.
/// In reality, this probably isn't want you want because entities
/// can be pushed away, for example. So, better approach might be
/// to do something to help update the intended direction of a unit
/// and stick to that
pub fn update_facing_direction<C: Component>(
    mut q_animation: Query<
        (
            &CharacterFacing,
            &mut AnimationFacing,
            &mut AnimationConfig,
            &mut Sprite,
            &LinearVelocity,
        ),
        (With<C>, Changed<CharacterFacing>),
    >,
) {
    for (facing, mut anim_facing, mut anim_config, mut sprite, velo) in &mut q_animation {
        // TODO: Move this elsewhere
        if velo.0 == Vec2::ZERO {
            anim_config.frame_timer.pause()
        } else if anim_config.frame_timer.is_paused() {
            anim_config.frame_timer.unpause()
        }
        anim_facing.update_facing(facing.c_dir, &mut anim_config, &mut sprite)
    }
}
