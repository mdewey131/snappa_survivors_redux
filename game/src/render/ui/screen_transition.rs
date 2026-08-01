use bevy::{ecs::system::SystemId, prelude::*, ui::FocusPolicy};

/// A screen transition slowly blacks out the screen, then fades back in
///
/// once it hits maximum fade, it will execute whatever you put in the
/// callback field. This can be used to, e.g. , transition game
/// state at the peak, so that underlying scene transition can happen
/// while hidden from the user
pub fn create_screen_transition(callback: Option<SystemId>) -> impl Scene {
    bsn! {
        #ScreenTransition
        ScreenTransition::new(callback)
        template_value(FocusPolicy::Block)
        Node {
            width: percent(100),
            height: percent(100)
        }
        BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.0))
        ZIndex(5)
    }
}

/// The ScreenTransition allows you put a cover over the current screen so that you can
/// do whatever you need to do in the meantime. This can be used to bring us into loading screens, the lobby, etc
#[derive(Component, Clone, Debug, Default)]
pub struct ScreenTransition {
    callback: Option<SystemId>,
    timer: Timer,
    /// Is this thing currently fading in or fading out?
    fade_in: bool,
}
impl ScreenTransition {
    fn new(callback: Option<SystemId>) -> Self {
        Self {
            callback,
            timer: Timer::from_seconds(0.5, TimerMode::Once),
            fade_in: true,
        }
    }
}

pub struct ScreenTransitionPlugin;
impl Plugin for ScreenTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
    }
}

/// Uses a linear fade at the moment
fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut q_timer: Query<(Entity, &mut ScreenTransition, &mut BackgroundColor)>,
) {
    for (ent, mut transition, mut bg) in &mut q_timer {
        transition.timer.tick(time.delta());
        let to_set = if transition.fade_in {
            transition.timer.fraction()
        } else {
            transition.timer.fraction_remaining()
        };
        bg.set_alpha(to_set);

        if transition.timer.just_finished() {
            if !transition.fade_in {
                commands.entity(ent).despawn()
            } else {
                transition.timer.reset();
                transition.fade_in = false;
                if let Some(sys) = transition.callback {
                    commands.run_system(sys)
                }
            }
        }
    }
}
