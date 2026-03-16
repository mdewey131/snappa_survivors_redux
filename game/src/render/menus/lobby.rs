use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    render::ui::button::*,
    shared::{
        game_kinds::{CurrentGameKind, GameKinds},
        game_rules::{Difficulty, GameRuleField, MapKind},
        players::CharacterKind,
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
fn lobby_node() -> Node {
    Node {
        display: Display::Flex,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::FlexStart,
        flex_direction: FlexDirection::Column,
        ..default()
    }
}

#[derive(Component, Debug, Clone, Copy)]
#[require(Node = lobby_top_rail())]
pub struct LobbyTopRail;
fn lobby_top_rail() -> Node {
    Node {
        display: Display::Flex,
        width: Val::Percent(100.0),
        height: Val::Percent(10.0),
        justify_content: JustifyContent::FlexStart,
        flex_direction: FlexDirection::Row,
        ..default()
    }
}

#[derive(Component, Debug, Clone, Copy)]
#[require(Node = lobby_main())]
pub struct LobbyMainContainer;
fn lobby_main() -> Node {
    Node {
        display: Display::Flex,
        width: Val::Percent(100.0),
        height: Val::Percent(90.0),
        justify_content: JustifyContent::SpaceAround,
        flex_direction: FlexDirection::Row,
        ..default()
    }
}

#[derive(Component, Debug, Clone, Copy)]
#[require(Node = lobby_subcontainer(20.0, 100.0))]
pub struct LobbyPlayerInfoContainer;

#[derive(Component, Debug, Clone, Copy)]
#[require(Node = lobby_subcontainer(50.0, 100.0))]
pub struct LobbyCharacterSelection;

#[derive(Component, Debug, Clone, Serialize, Deserialize, Default)]
#[relationship_target(relationship = SelectedCharacterButton)]
pub struct CharacterSelectionButton {
    pub kind: CharacterKind,
    #[relationship]
    selected_by: Vec<Entity>,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[relationship(relationship_target = CharacterSelectionButton)]
pub struct SelectedCharacterButton(Entity);

#[derive(Component, Debug, Clone, Copy)]
#[require(Node = lobby_subcontainer(20.0, 100.0))]
pub struct LobbySettingsSection;

fn lobby_subcontainer(width: f32, height: f32) -> Node {
    Node {
        display: Display::Flex,
        width: Val::Percent(width),
        height: Val::Percent(height),
        justify_content: JustifyContent::SpaceAround,
        flex_direction: FlexDirection::Column,
        ..default()
    }
}

#[derive(Component, Debug, Clone, Copy)]
#[require(Node = lobby_setting(100.0, 30.0))]
pub struct LobbyDifficulty;

fn lobby_setting(width: f32, height: f32) -> Node {
    Node {
        display: Display::Flex,
        width: Val::Percent(width),
        height: Val::Percent(height),
        justify_content: JustifyContent::SpaceAround,
        flex_direction: FlexDirection::Row,
        ..default()
    }
}
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

fn make_lobby(mut commands: Commands, assets: Res<AssetServer>) {
    let lobby = commands
        .spawn((LobbyScreen, DespawnOnExit(AppState::Lobby)))
        .id();

    // Lobby top rail
    let lobby_top = commands.spawn((LobbyTopRail, ChildOf(lobby))).id();
    commands.spawn((ChildOf(lobby_top), ContainerLobbyBackButton));

    // Lobby Main Body
    let lobby_main = commands.spawn((LobbyMainContainer, ChildOf(lobby))).id();

    let player_info = commands
        .spawn((LobbyPlayerInfoContainer, ChildOf(lobby_main)))
        .id();
    let character_selection = commands
        .spawn((LobbyCharacterSelection, ChildOf(lobby_main)))
        .id();
    let settings_section = commands
        .spawn((LobbySettingsSection, ChildOf(lobby_main)))
        .id();

    let diff_section = commands
        .spawn((LobbyDifficulty, ChildOf(settings_section)))
        .id();

    for diff in [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard].iter() {
        let color = match *diff {
            Difficulty::Easy => Color::srgb(0.5, 0.5, 0.9),
            Difficulty::Normal => Color::srgb(0.5, 0.9, 0.5),
            Difficulty::Hard => Color::srgb(0.9, 0.5, 0.5),
        };
        let button = GameButton::new(GameButtonOnRelease::EventTrigger);
        let style = GameButtonStyle::default()
            .with_text(format!("{:?}", diff))
            .with_color(color);
        let system =
            commands.register_system(crate::shared::game_rules::send_game_change_message_callback);
        let cb = CallbackWithInput::<In<Difficulty>>(system);

        let btn_ent = button.spawn(&mut commands, &assets, style);
        commands.entity(btn_ent).insert((
            ChangeGameSettingButton(*diff),
            ChildOf(diff_section),
            cb,
        ));
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
