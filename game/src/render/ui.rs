use bevy::{ecs::component::Mutable, prelude::*};

pub mod button;
pub mod screen_transition;
use button::GameButtonPlugin;
use screen_transition::ScreenTransitionPlugin;

pub struct SharedUIPlugin;

impl Plugin for SharedUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((GameButtonPlugin, ScreenTransitionPlugin))
            .add_systems(
                Update,
                (
                    ui_fade_system::<Sprite>,
                    ui_fade_system::<BackgroundColor>,
                    ui_fade_system::<ImageNode>,
                ),
            )
            .add_observer(trigger_ui_fade_set_color::<Sprite>)
            .add_observer(trigger_ui_fade_set_color::<BackgroundColor>)
            .add_observer(trigger_ui_fade_set_color::<ImageNode>);
    }
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct FadeEffect {
    pub timer: Timer,
    pub fade_in: bool,
    pub ease: EaseFunction,
}

impl FadeEffect {
    pub fn fade_out(time: f32, ease: EaseFunction) -> Self {
        Self {
            timer: Timer::from_seconds(time, TimerMode::Once),
            fade_in: false,
            ease,
        }
    }

    pub fn fade_in(time: f32, ease: EaseFunction) -> Self {
        Self {
            timer: Timer::from_seconds(time, TimerMode::Once),
            fade_in: true,
            ease,
        }
    }
}

pub trait CanFade {
    fn set_alpha(&mut self, percent: f32);
}
impl CanFade for ImageNode {
    fn set_alpha(&mut self, percent: f32) {
        self.color.set_alpha(percent);
    }
}
impl CanFade for BackgroundColor {
    fn set_alpha(&mut self, percent: f32) {
        self.0.set_alpha(percent);
    }
}
impl CanFade for Sprite {
    fn set_alpha(&mut self, percent: f32) {
        self.color.set_alpha(percent)
    }
}

/// Should help to prevent with cases where something spawns in and is super visible immediately despite needing to fade in
fn trigger_ui_fade_set_color<C: Component<Mutability = Mutable> + CanFade>(
    t: On<Add, FadeEffect>,
    mut q_node: Query<(&mut C, &FadeEffect)>,
) {
    if let Ok((mut comp, eff)) = q_node.get_mut(t.entity) {
        if eff.fade_in {
            comp.set_alpha(0.0);
        }
    }
}

fn ui_fade_system<C: Component<Mutability = Mutable> + CanFade>(
    time: Res<Time>,
    mut q_sprite: Query<(&mut FadeEffect, &mut C)>,
) {
    for (mut eff, mut comp) in &mut q_sprite {
        eff.timer.tick(time.delta());
        let percent = if eff.fade_in {
            let timer_pct = eff.timer.fraction();
            eff.ease.sample(timer_pct).unwrap()
        } else {
            let timer_pct = eff.timer.fraction_remaining();
            eff.ease.sample(timer_pct).unwrap()
        };
        comp.set_alpha(percent);
    }
}
