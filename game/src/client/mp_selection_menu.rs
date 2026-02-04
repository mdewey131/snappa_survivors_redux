use bevy::prelude::*;

use crate::{
    client::game_client::GameClientConfig, render::ui::button::*, shared::states::AppState,
    utils::CallbackWithInput,
};

pub struct MPSelectionMenuPlugin;

impl Plugin for MPSelectionMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MultiplayerServerSelection), spawn_menu)
            .add_observer(trigger_connection)
            .add_observer(back_to_menu_button);
    }
}

#[derive(Component)]
#[require(Node = server_menu_node())]
pub struct ServerMenuScreen;
fn server_menu_node() -> Node {
    Node {
        display: Display::Grid,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        grid_template_columns: vec![
            RepeatedGridTrack::percent(1, 10.0),
            RepeatedGridTrack::percent(1, 80.0),
            RepeatedGridTrack::percent(1, 10.0),
        ],
        grid_template_rows: vec![
            RepeatedGridTrack::percent(1, 10.0),
            RepeatedGridTrack::percent(1, 80.0),
            RepeatedGridTrack::percent(1, 10.0),
        ],
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

#[derive(Component, Debug)]
#[require(Node = main_menu_button())]
pub struct BacktoMainMenuButton;
fn main_menu_button() -> Node {
    Node {
        grid_column: GridPlacement::start(1),
        grid_row: GridPlacement::start(1),
        ..default()
    }
}

#[derive(Component, Debug)]
pub struct ServerConnectionButton;
fn server_connect_button() -> Node {
    Node {
        grid_column: GridPlacement::start(2),
        grid_row: GridPlacement::start(2),
        justify_self: JustifySelf::Center,
        ..default()
    }
}

fn spawn_menu(mut commands: Commands, assets: Res<AssetServer>) {
    let screen = commands
        .spawn((
            ServerMenuScreen,
            DespawnOnExit(AppState::MultiplayerServerSelection),
        ))
        .id();

    let back_btn = GameButton::new(GameButtonOnRelease::EventTrigger);
    let back_button_style = GameButtonStyle::new(GameButtonImage::default());
    let back_button = back_btn.spawn(&mut commands, &assets, back_button_style);
    commands.entity(back_button).insert((
        BacktoMainMenuButton,
        main_menu_button(),
        Text::from("<="),
        ChildOf(screen),
    ));

    let server_con_button = GameButton::new(GameButtonOnRelease::EventTrigger);
    let button_style = GameButtonStyle::new(GameButtonImage::default())
        .with_color(Color::srgb(1.0, 0.0, 0.0))
        .with_text("Connect".into());

    let btn_ent = server_con_button.spawn(&mut commands, &assets, button_style);

    let system = commands.register_system(super::transition_to_multi_player);
    let callback_system = crate::utils::CallbackWithInput::<In<GameClientConfig>>(system);

    commands.entity(btn_ent).insert((
        ServerConnectionButton,
        server_connect_button(),
        callback_system,
        ChildOf(screen),
    ));
}

fn trigger_connection(
    t: On<ButtonReleased>,
    mut commands: Commands,
    q_callback: Query<&CallbackWithInput<In<GameClientConfig>>, With<GameButton>>,
) {
    if let Ok(cb) = q_callback.get(t.entity) {
        commands.run_system_with(cb.0, GameClientConfig::new_with_random_c_id());
    }
}

fn back_to_menu_button(
    trigger: On<ButtonReleased>,
    mut state: ResMut<NextState<AppState>>,
    q_button: Query<&BacktoMainMenuButton>,
) {
    if let Ok(_btn) = q_button.get(trigger.entity) {
        state.set(AppState::MainMenu);
    }
}
