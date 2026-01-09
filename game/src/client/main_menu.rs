use bevy::prelude::*;

use crate::{
    render::ui::{button::*, screen_transition::ScreenTransition},
    shared::{despawn_timer::DespawnTimer, states::AppState},
};
#[derive(Component, Debug, Clone, Copy)]
#[require(Node = node_main_menu_screen())]
pub struct MainMenuScreen;

fn node_main_menu_screen() -> Node {
    Node {
        height: Val::Percent(100.0),
        width: Val::Percent(100.0),
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::SpaceAround,
        align_items: AlignItems::Center,
        ..default()
    }
}

#[derive(Component, Debug, Clone, Copy)]
#[require(Node = node_title())]
pub struct TitleRegion;

fn node_title() -> Node {
    Node {
        width: Val::Percent(80.0),
        height: Val::Percent(40.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

#[derive(Component, Debug, Clone, Copy)]
#[require(Node = node_mm_button_well())]
struct ButtonWell;

fn node_mm_button_well() -> Node {
    Node {
        width: Val::Percent(80.0),
        height: Val::Percent(40.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        ..default()
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ButtonSinglePlayerGame;

#[derive(Component, Debug, Clone, Copy)]
pub struct OpenSettingsButton;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_buttons)
            .add_systems(OnEnter(AppState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(AppState::MainMenu), despawn_main_menu);
    }
}

fn register_buttons(world: &mut World) {
    let start_1player_sys = world.register_system(super::transition_to_single_player);
    //let open_settings_sys = world.register_system(open_settings);
    let mut button_systems = world.resource_mut::<ButtonSystems>();

    (*button_systems).insert("start_1_player".into(), start_1player_sys);
    //(*button_systems).insert("open_settings".into(), open_settings_sys);
}

/*

fn open_settings(mut commands: Commands, assets: Res<AssetServer>, systems: Res<ButtonSystems>) {
    spawn_settings_screen(&mut commands, &assets, &systems);
}
*/

fn spawn_main_menu(mut commands: Commands, assets: Res<AssetServer>, systems: Res<ButtonSystems>) {
    let main_menu_screen = commands
        .spawn((
            MainMenuScreen,
            ImageNode::new(assets.load("main_menu/main_menu_image.png")),
            // Way to the back now, stop blocking the bevy inspector UI
        ))
        .id();

    // Title segment
    let _title_well = commands
        .spawn((TitleRegion, ChildOf(main_menu_screen)))
        .with_child(Text::new("Snappa Survivors"));

    // Spawning the button well
    let button_well = commands.spawn((ButtonWell, ChildOf(main_menu_screen))).id();

    let start_game_sys = systems.get("start_1_player").unwrap();

    let start_1_player_button =
        GameButton::new(GameButtonOnRelease::TriggerSystem(*start_game_sys));
    let start_button_style = GameButtonStyle::new(GameButtonImage::default())
        .with_color(Color::srgb(1.0, 0.0, 0.0))
        .with_text("Start Game".into());

    let st_btn_ent = start_1_player_button.spawn(&mut commands, &assets, start_button_style);

    commands
        .entity(st_btn_ent)
        .insert((ButtonSinglePlayerGame, ChildOf(button_well)));

    /*
    let open_settings_sys = systems.get("open_settings").unwrap();
    let set_btn = GameButton::new(GameButtonOnRelease::TriggerSystem(*open_settings_sys));
    let set_btn_style = GameButtonStyle::new(GameButtonImage::default())
        .with_color(Color::srgb(0.8, 0.4, 0.4))
        .with_text("Settings".into());

    let set_btn_ent = set_btn.spawn(&mut commands, &assets, set_btn_style);
    commands
        .entity(set_btn_ent)
        .insert((OpenSettingsButton, ChildOf(button_well)));
    */
}

fn despawn_main_menu(mut commands: Commands, menu: Single<Entity, With<MainMenuScreen>>) {
    commands.entity(*menu).insert((DespawnTimer::new(0.5),));

    ScreenTransition::new(&mut commands);
}
