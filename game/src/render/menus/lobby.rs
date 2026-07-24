use bevy::{picking::hover::Hovered, platform::collections::HashMap, prelude::*};
use lightyear::prelude::{Client, Controlled, MessageSender};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

pub const BORDER_WIDTH: f32 = 20.0;

use crate::{
    render::ui::{button::*, picking::SelectedBy},
    shared::{
        GameMainChannel,
        game_kinds::{CurrentGameKind, GameKinds, SinglePlayer, is_single_player},
        game_rules::{Difficulty, GameRuleField},
        lobby::{ClientChangeCharacterMessage, PlayerInLobby},
        maps::MapKind,
        players::CharacterKind,
        states::AppState,
    },
    utils::{AssetFolder, CallbackWithInput},
};

pub struct LobbyMenuPlugin;

impl Plugin for LobbyMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Lobby), make_lobby)
            .add_systems(
                Update,
                (mp_propagate_client_change_character_message_to_server
                    .run_if(in_state(AppState::Lobby).and_then(not(is_single_player)))),
            )
            .add_observer(trigger_game_change_message_callback::<Difficulty>)
            .add_observer(trigger_game_change_message_callback::<MapKind>);
    }
}

fn lobby() -> impl Scene {
    bsn! {
        #LobbyScreen
        LobbyScreen
        BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 1.0))
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            border: UiRect::all(px(10.0)),
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Column,
        }
        DespawnOnExit<AppState>(AppState::Lobby)
        Children[
            (
                #TopRail
                LobbyTopRail
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(10.0),
                    justify_content: JustifyContent::FlexStart,
                    border: UiRect::all(px(10.0))
                }
                BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 1.0))
                Children[
                    LobbyBackButton
                    BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 1.0))
                    Node {
                        width: px(85),
                        height: px(85)
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(px(10.0))
                    }
                ]

            ),
            lobby_main()
        ]
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct LobbyScreen;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct LobbyTopRail;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct LobbyMainContainer;

fn lobby_main() -> impl Scene {
    bsn! {
        #LobbyMainContainer
        LobbyMainContainer
        Node {
                display: Display::Flex,
                width: Val::Percent(100.0),
                height: Val::Percent(90.0),
                justify_content: JustifyContent::SpaceAround,
                flex_direction: FlexDirection::Row,
                border: UiRect::all(px(10.0))
        }
        BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 1.0))
        Children[
            player_info_well(),
            character_selection_well(),
        ]
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct LobbyPlayerInfoWell;

fn player_info_well() -> impl Scene {
    bsn! {
        #PlayerInfoWell
        LobbyPlayerInfoWell
        Node {
            width: percent(15),
            height: percent(100),
            border: UiRect::all(px(10.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center
        }
        BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 1.0))
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct LobbyCharacterSelection;
fn character_selection_well() -> impl Scene {
    bsn! {
        #CharacterSelectionWell
        LobbyCharacterSelection
        Node {
            width: percent(50.0),
            height: percent(100),
            border: UiRect::all(px(10.0)),
            flex_direction: FlexDirection::Column
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch
        }
        BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 1.0))
        Children[
            char_button_container(),
            (
                #LobbyCharacterPreview
                LobbyCharacterPreview {m_char: None}
                BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 1.0))
                Node {
                   height: percent(30)
                   justify_content: JustifyContent::Center,
                   border: UiRect::all(px(10.0))
                   align_items: AlignItems::Center,
                }
            )
        ]
    }
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct CharacterSelectionButtonContainer;

pub fn char_button_container() -> impl Scene {
    bsn! {
        #ButtonContainer
        CharacterSelectionButtonContainer
        BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 1.0))
        Node {
            height: percent(70),
            border: UiRect::all(px(10.0)),
            justify_content: JustifyContent::SpaceAround,
            align_items: AlignItems::Center,
            flex_wrap: FlexWrap::Wrap
        }
        Children[
            char_button(CharacterKind::Dewey),
            char_button(CharacterKind::Matthew),
            char_button(CharacterKind::Mark),
            char_button(CharacterKind::Ryan),
            char_button(CharacterKind::Shaunt),
            char_button(CharacterKind::Gabe),
            char_button(CharacterKind::Paul),
            char_button(CharacterKind::Finn),
        ]
    }
}

#[derive(Component, Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub struct CharacterSelectionButton {
    char: CharacterKind,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterSelectionIcon;

#[derive(Component, Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterSelectionText;

fn char_button(c: CharacterKind) -> impl Scene {
    let folder = AssetFolder::from(c);
    let image = format!("{}/lobby_portrait.png", folder.0);
    let name: String = c.into();

    bsn! {
        #CharacterButton
        CharacterSelectionButton {char: c}
        SelectedBy(vec![])
        Button
        Pickable {
                should_block_lower: true,
                is_hoverable: true,
            }
        Node {
            width: px(125),
            height: px(125),
            border: UiRect::all(px(10.0))
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Stretch
        }
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.5))
        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.2))
        on(hover_change_preview_character)
        on(on_leave_remove_character_preview)
        on(click_select_character)
        Children[
            (
                #CharacterIcon
                CharacterSelectionIcon
                ImageNode {image}
                Node {
                    height: percent(80),
                    border: UiRect::all(px(10.0))
                }
            ),
            (
                #CharacterText
                CharacterSelectionText
                Text::new(name)
            )
        ]
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct LobbyCharacterPreview {
    m_char: Option<CharacterKind>,
}

fn change_preview_character(
    commands: &mut Commands,
    preview: &mut Single<(Entity, &mut LobbyCharacterPreview)>,
    to: Option<CharacterKind>,
) {
    if let Some(old_character) = preview.1.m_char {
        commands.entity(preview.0).despawn_children();
    }
    if let Some(c) = to {
        commands.spawn_scene_list(lobby_character_preview(preview.0, c));
    }
    preview.1.m_char = to;
}

fn lobby_character_preview(on: Entity, c: CharacterKind) -> impl SceneList {
    let folder = AssetFolder::from(c);
    let description: String = c.into();
    let image = format!("{}/lobby_portrait.png", folder.0);
    bsn_list![
        (
            #Portrait
            ChildOf(on)
            LobbyCharacterPortrait
            BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 1.0))
            ImageNode {image}
            Node {
                width: percent(20),
                border: UiRect::all(px(10.0))
            }
        ),
        (
            #Description
            ChildOf(on)
            BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 1.0))
            LobbyCharacterDescription
            Text::from(description)
            Node {
                width: percent(80),
                border: UiRect::all(px(10.0))
            }
        )
    ]
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct LobbyCharacterDescription;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct LobbyCharacterPortrait;

fn hover_change_preview_character(
    on: On<Pointer<Over>>,
    mut commands: Commands,
    q_hovered: Query<&CharacterSelectionButton>,
    mut preview: Single<(Entity, &mut LobbyCharacterPreview)>,
) {
    info!("Hovered!");
    if let Ok(button) = q_hovered.get(on.entity) {
        change_preview_character(&mut commands, &mut preview, Some(button.char));
    }
}

fn on_leave_remove_character_preview(
    on: On<Pointer<Leave>>,
    mut commands: Commands,
    q_hovered: Query<(&CharacterSelectionButton, &SelectedBy)>,
    mut preview: Single<(Entity, &mut LobbyCharacterPreview)>,
    q_player: Query<
        (),
        (
            With<PlayerInLobby>,
            Or<(With<SinglePlayer>, With<Controlled>)>,
        ),
    >,
) {
    info!("Left!");
    if let Ok((_button, selected)) = q_hovered.get(on.entity) {
        let player_selected = selected.0.iter().any(|ent| q_player.get(*ent).is_ok());
        if !player_selected {
            change_preview_character(&mut commands, &mut preview, None);
        }
    }
}

fn click_select_character(
    on: On<Pointer<Press>>,
    mut local: MessageWriter<ClientChangeCharacterMessage>,
    mut q_pressed: Query<(&CharacterSelectionButton, &mut SelectedBy)>,
    player: Single<
        Entity,
        (
            With<PlayerInLobby>,
            Or<(With<Controlled>, With<SinglePlayer>)>,
        ),
    >,
) {
    if q_pressed.get(on.entity).is_err() {
        return;
    }
    info!("Clicked!");

    for (_button, mut selected) in &mut q_pressed {
        selected.0.retain(|sel| *sel != *player)
    }

    let (button, mut selected) = q_pressed.get_mut(on.entity).unwrap();
    selected.0.push(*player);
    local.write(ClientChangeCharacterMessage { char: button.char });
}

#[derive(Component, Debug, Clone, Copy)]
pub struct LobbySettingsSection;

#[derive(Component, Debug, Clone, Copy)]
pub struct LobbyDifficulty;

/// The node that contains a button to go back to the previous screen.
/// We just store this as a holder because we want to selectively spawn this
/// button depending on the environment (clients should have this, servers should not)
#[derive(Component, Debug, Clone, Copy)]
pub struct ContainerLobbyBackButton;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct LobbyBackButton;

#[derive(Component)]
pub struct ChangeGameSettingButton<F: GameRuleField>(F);

fn make_lobby(mut commands: Commands, assets: Res<AssetServer>, _game_kind: Res<CurrentGameKind>) {
    commands.spawn_scene(lobby());
    /*
        // TODO: Change the callback to work regardless of the game type (single or multiplayer)
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
    */
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
/// We spawn the container for the button on either interface
/// for the sake of getting layour correct,
/// so actual button can be selectively spawned
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

fn mp_propagate_client_change_character_message_to_server(
    mut reader: MessageReader<ClientChangeCharacterMessage>,
    mut q_client: Single<&mut MessageSender<ClientChangeCharacterMessage>>,
) {
    for e in reader.read() {
        q_client.send::<GameMainChannel>(*e);
    }
}
