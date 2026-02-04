use crate::{render::ui::button::*, shared::states::AppState};
use bevy::prelude::*;
use lightyear::prelude::*;

#[derive(Component)]
#[require(Node = screen())]
pub struct PauseMenuScreen;
fn screen() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

#[derive(Component)]
#[require(Node = menu())]
pub struct PauseMenu;
fn menu() -> Node {
    Node {
        width: Val::Percent(40.0),
        height: Val::Percent(40.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        flex_direction: FlexDirection::Column,
        ..default()
    }
}

pub struct ExitGameButton;

pub fn spawn_pause_menu(mut commands: Commands, assets: Res<AssetServer>) {
    let screen = commands
        .spawn((PauseMenuScreen, DespawnOnExit(AppState::InGame)))
        .id();

    let menu = commands.spawn((PauseMenu, ChildOf(screen))).id();
    let sys_id = commands.register_system(exit_game);
    let button = GameButton::new(GameButtonOnRelease::TriggerSystem((sys_id)));
    let style = GameButtonStyle::default()
        .with_color(Color::srgb(1.0, 0.0, 0.0))
        .with_text(String::from("Exit Game"));
    let btn_entity = button.spawn(&mut commands, &assets, style);
    commands.entity(btn_entity).insert(ChildOf(menu));
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
