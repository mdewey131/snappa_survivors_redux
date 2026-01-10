use bevy::prelude::*;

use crate::{
    client::game_client::GameClientConfig, render::ui::button::*, shared::states::AppState,
    utils::CallbackWithInput,
};

pub struct MPSelectionMenuPlugin;

impl Plugin for MPSelectionMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MultiplayerServerSelection), spawn_menu)
            .add_observer(trigger_connection);
    }
}

#[derive(Component, Debug)]
pub struct ServerConnectionButton;

#[derive(Component)]
#[require(Node = Node::default())]
pub struct ServerMenuScreen;

fn spawn_menu(mut commands: Commands, assets: Res<AssetServer>) {
    let screen = commands
        .spawn((
            ServerMenuScreen,
            DespawnOnExit(AppState::MultiplayerServerSelection),
        ))
        .id();
    let server_con_button = GameButton::new(GameButtonOnRelease::EventTrigger);
    let button_style = GameButtonStyle::new(GameButtonImage::default())
        .with_color(Color::srgb(1.0, 0.0, 0.0))
        .with_text("Connect".into());

    let btn_ent = server_con_button.spawn(&mut commands, &assets, button_style);

    let system = commands.register_system(super::transition_to_multi_player);
    let callback_system = crate::utils::CallbackWithInput::<In<GameClientConfig>>(system);

    commands
        .entity(btn_ent)
        .insert((ServerConnectionButton, callback_system, ChildOf(screen)));
}

fn trigger_connection(
    t: On<ButtonReleased>,
    mut commands: Commands,
    q_callback: Query<&CallbackWithInput<In<GameClientConfig>>, With<GameButton>>,
) {
    if let Ok(cb) = q_callback.get(t.entity) {
        commands.run_system_with(cb.0, GameClientConfig::SINGLE_PLAYER);
    }
}
