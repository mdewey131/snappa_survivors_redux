use bevy::prelude::*;

use crate::{
    render::ui::FadeEffect,
    shared::{loading::LevelLoadingState, states::AppState},
};

pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(LevelLoadingState::LevelLoading),
            popup_loading_screen,
        );
    }
}

#[derive(Component, Debug, Clone)]
#[require(
    Node = full_screen(),
    BackgroundColor = BackgroundColor(Color::srgba(0.05,0.05,0.05,0.0)),
    FadeEffect = FadeEffect::fade_in(0.1, EaseFunction::CubicOut)
)]
pub struct LoadingScreen;
fn full_screen() -> Node {
    Node {
        height: Val::Percent(100.0),
        width: Val::Percent(100.0),
        display: Display::Flex,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

#[derive(Component, Debug, Clone, Copy)]
#[require(Text = Text::from("Loading..."))]
pub struct LoadingScreenText;

fn popup_loading_screen(mut commands: Commands) {
    commands
        .spawn((LoadingScreen, DespawnOnExit(AppState::LoadingLevel)))
        .with_child(LoadingScreenText );
}
