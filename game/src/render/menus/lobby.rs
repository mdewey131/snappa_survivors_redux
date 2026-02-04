use bevy::prelude::*;

use crate::{
    render::ui::button::*,
    shared::{
        game_kinds::{CurrentGameKind, GameKinds},
        game_rules::{Difficulty, GameRuleField, MapKind},
        states::AppState,
    },
    utils::CallbackWithInput,
};

pub struct LobbyMenuPlugin;

impl Plugin for LobbyMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Lobby), make_lobby)
            .add_observer(trigger_game_change_message_callback::<Difficulty>)
            .add_observer(trigger_game_change_message_callback::<MapKind>);
    }
}

#[derive(Component, Debug, Clone, Copy)]
#[require(Node = lobby_node())]
pub struct LobbyScreen;

/// The node that contains a button to go back to the previous screen.
/// We just store this as a holder because we want to selectively spawn this
/// button depending on the environment (clients should have this, servers should not)
#[derive(Component, Debug, Clone, Copy)]
#[require(Node = Node::default())]
pub struct ContainerLobbyBackButton;

#[derive(Component, Debug, Clone, Copy)]
pub struct LobbyBackButton;

#[derive(Component)]
pub struct ChangeGameSettingButton<F: GameRuleField>(F);

fn lobby_node() -> Node {
    Node {
        display: Display::Flex,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::SpaceAround,
        ..default()
    }
}

fn make_lobby(mut commands: Commands, assets: Res<AssetServer>) {
    let lobby = commands
        .spawn((LobbyScreen, DespawnOnExit(AppState::Lobby)))
        .id();
    commands.spawn((ChildOf(lobby), ContainerLobbyBackButton));

    for diff in [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard].iter() {
        let button = GameButton::new(GameButtonOnRelease::EventTrigger);
        let style = GameButtonStyle::default().with_text(format!("{:?}", diff));
        let system =
            commands.register_system(crate::shared::game_rules::send_game_change_message_callback);
        let cb = CallbackWithInput::<In<Difficulty>>(system);

        let btn_ent = button.spawn(&mut commands, &assets, style);
        commands
            .entity(btn_ent)
            .insert((ChangeGameSettingButton(*diff), ChildOf(lobby), cb));
    }
}

fn trigger_game_change_message_callback<F: GameRuleField>(
    t: On<ButtonReleased>,
    mut commands: Commands,
    q_cb: Query<(&CallbackWithInput<In<F>>, &ChangeGameSettingButton<F>)>,
) {
    if let Ok((cb, button)) = q_cb.get(t.entity) {
        commands.run_system_with(cb.0, button.0);
    }
}

pub fn spawn_lobby_back_button(
    trigger: On<Add, ContainerLobbyBackButton>,
    mut commands: Commands,
    game_kind: Res<CurrentGameKind>,
    assets: Res<AssetServer>,
) {
    let btn = GameButton::new(GameButtonOnRelease::EventTrigger);
    let text = match game_kind.0.unwrap() {
        GameKinds::MultiPlayer => "Back to Server Selection".into(),
        GameKinds::SinglePlayer => "Back to Main Menu".into(),
    };
    let style = GameButtonStyle::default().with_text(text);
    let button = btn.spawn(&mut commands, &assets, style);

    commands
        .entity(button)
        .insert((ChildOf(trigger.entity), LobbyBackButton));
}
