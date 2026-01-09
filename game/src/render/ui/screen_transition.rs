use crate::render::ui::FadeEffect;
use bevy::prelude::*;

/// The ScreenTransition allows you put a cover over the current screen so that you can
/// do whatever you need to do in the meantime. This can be used to bring us into loading screens, the lobby, etc
#[derive(Component, Clone, Copy, Debug)]
#[require(
    Node = node_transition(),
    FadeEffect = FadeEffect::fade_in(0.5, EaseFunction::Linear),
    BackgroundColor = BackgroundColor(Color::BLACK),
    ZIndex = ZIndex(5)
)]
pub struct ScreenTransition;

pub struct ScreenTransitionPlugin;
impl Plugin for ScreenTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_timer_up, despawn));
    }
}

fn node_transition() -> Node {
    Node {
        height: Val::Percent(100.0),
        width: Val::Percent(100.0),
        ..default()
    }
}

impl ScreenTransition {
    pub fn new(commands: &mut Commands) -> Entity {
        commands.spawn(ScreenTransition).id()
    }
}

fn handle_timer_up(
    mut commands: Commands,
    q_timer: Query<(Entity, &FadeEffect), With<ScreenTransition>>,
) {
    for (ent, fade) in &q_timer {
        if fade.timer.is_finished() && fade.fade_in {
            commands
                .entity(ent)
                .insert(FadeEffect::fade_out(0.5, EaseFunction::Linear));
        }
    }
}

fn despawn(mut commands: Commands, q_timer: Query<(Entity, &FadeEffect), With<ScreenTransition>>) {
    for (ent, fade) in &q_timer {
        if fade.timer.is_finished() && !fade.fade_in {
            commands.entity(ent).despawn();
        }
    }
}
