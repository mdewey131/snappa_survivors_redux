use crate::{render::ui::button::*, shared::states::AppState};
use bevy::prelude::*;
use lightyear::prelude::*;

fn pause_menu() -> impl Scene {
    bsn! {
        #PauseMenuScreen
        PauseMenuScreen
        Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
        }
        DespawnOnExit<AppState>(AppState::InGame)
        Children[
            menu()
        ]
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PauseMenuScreen;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PauseMenu;
fn menu() -> impl Scene {
    bsn! {
        #PauseMenu
        PauseMenu
        Node {
            width: Val::Percent(40.0),
            height: Val::Percent(40.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            border: UiRect::all(px(10))
        }
        BackgroundColor(Color::srgba(1.0, 0.9, 0.6, 1.0))
        BorderColor::all(Color::srgba(0.8, 0.7, 0.4, 1.0))
        Children[
           (
               #ExitGameButton
               Node {
                   width: percent(80),
                   height: percent(20),
                   border: UiRect::all(px(5)),
                   justify_content: JustifyContent::Center,
                   align_items: AlignItems::Center,
               }
               game_button("Exit Game", Some(Color::srgb(0.3, 0.3, 0.3)), Some(Color::WHITE) )
               on(|_t: On<Pointer<Press>>, mut next_state: ResMut<NextState<AppState>>| next_state.set(AppState::MainMenu))
           )
        ]
    }
}

pub struct ExitGameButton;

pub fn spawn_pause_menu(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn_scene(pause_menu());
    /*
    let screen = commands
        .spawn((PauseMenuScreen, DespawnOnExit(AppState::InGame)))
        .id();

    let menu = commands.spawn((PauseMenu, ChildOf(screen))).id();
    let sys_id = commands.register_system(exit_game);
    let button = GameButton::new(GameButtonOnRelease::TriggerSystem(sys_id));
    let style = GameButtonStyle::default()
        .with_color(Color::srgb(1.0, 0.0, 0.0))
        .with_text(String::from("Exit Game"));
    let btn_entity = button.spawn(&mut commands, &assets, style);
    commands.entity(btn_entity).insert(ChildOf(menu));
    */
}

pub fn despawn_pause_menu(mut commands: Commands, q_menu: Single<Entity, With<PauseMenuScreen>>) {
    commands.entity(*q_menu).despawn();
}

fn exit_game(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    q_client: Option<Single<Entity, With<Client>>>,
) {
    if let Some(q_c) = q_client {
        commands.entity(*q_c).despawn();
    }
    next_state.set(AppState::MainMenu);
}
