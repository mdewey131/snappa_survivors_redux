use bevy::prelude::*;

#[cfg(feature = "dev")]
use crate::utils::zoo::*;
use crate::{
    client::set_app_state_to_lobby,
    render::ui::{
        button::*,
        screen_transition::{ScreenTransition, create_screen_transition},
    },
    shared::{
        despawn_timer::DespawnTimer,
        game_kinds::{self, CurrentGameKind},
        states::AppState,
    },
};

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct MainMenuScreen;

fn main_menu() -> impl Scene {
    bsn! {
        #MainMenu
        MainMenuScreen
        Node {
            width: percent(100),
            height: percent(100)
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceAround,
            align_items: AlignItems::Center,
        }
        ZIndex(1)
        ImageNode{ image: "main_menu/main_menu_image.png"}
        DespawnOnExit<AppState>(AppState::MainMenu)
        Children [
            title_region(),
            button_well()
        ]
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct TitleRegion;

fn title_region() -> impl Scene {
    bsn! {
        #Title
        TitleRegion
        Node {
            width: percent(80),
            height: percent(40),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        Children[
            Text::new("Snappa Survivors")
        ]
    }
}
#[derive(Component, Debug, Clone, Copy, Default)]
struct ButtonWell;

fn button_well() -> impl Scene {
    bsn! {
       #ButtonWell
       ButtonWell
       Node {
            width: percent(80),
            height: percent(40),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
        }
        Children [(
            #SinglePlayer
            ButtonSinglePlayerGame
            game_button("Single Player", None, None)
            on(on_press_move_to_single_player)
        )]
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ButtonSinglePlayerGame;

#[derive(Component, Debug, Clone, Copy)]
pub struct ButtonMultiPlayerGame;

#[derive(Component, Debug, Clone, Copy)]
pub struct OpenSettingsButton;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), spawn_main_menu);
        //.add_systems(OnExit(AppState::MainMenu), despawn_main_menu);
    }
}
/*

fn open_settings(mut commands: Commands, assets: Res<AssetServer>, systems: Res<ButtonSystems>) {
    spawn_settings_screen(&mut commands, &assets, &systems);
}
*/

fn spawn_main_menu(mut commands: Commands, assets: Res<AssetServer>, systems: Res<ButtonSystems>) {
    commands.spawn_scene(main_menu());
}

fn move_to_multiplayer_menu(mut state: ResMut<NextState<AppState>>) {
    state.set(AppState::MultiplayerServerSelection)
}

fn on_press_move_to_single_player(
    on: On<Pointer<Press>>,
    mut commands: Commands,
    mut game_kinds: ResMut<CurrentGameKind>,
) {
    game_kinds.0 = Some(game_kinds::GameKinds::SinglePlayer);
    let sys_on_transition = commands.register_system(set_app_state_to_lobby);
    commands.spawn_scene(create_screen_transition(Some(sys_on_transition)));
}
