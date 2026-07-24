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

fn loading_screen() -> impl Scene {
    bsn! {
        #LoadingScreen
        LoadingScreen
        BackgroundColor(Color::srgba(0.05,0.05,0.05, 1.0))
        DespawnOnExit<AppState>(AppState::LoadingLevel)
        Node {
            height: Val::Percent(100.0),
            width: Val::Percent(100.0),
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center
        }
        Children [
            LoadingScreenText
            Text("Loading...")
        ]
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct LoadingScreen;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct LoadingScreenText;

fn popup_loading_screen(mut commands: Commands) {
    commands.spawn_scene(loading_screen());
}
