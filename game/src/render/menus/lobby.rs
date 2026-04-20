use bevy::{platform::collections::HashMap, prelude::*};
use lightyear::prelude::{Client, MessageSender};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{
    render::ui::button::*,
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
                mp_propagate_client_change_character_message_to_server
                    .run_if(in_state(AppState::Lobby).and(not(is_single_player))),
            )
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

#[derive(Component, Debug, Clone)]
#[require(Node = character_selection_container(50.0, 100.0))]
pub struct LobbyCharacterSelection {
    pub buttons: HashMap<CharacterKind, Entity>,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Default)]
#[require(Node = char_sel_button(), Button = Button, Pickable = char_button_picking())]
pub struct CharacterSelectionButton {
    pub kind: CharacterKind,
    selected_by: Vec<Entity>,
}
fn char_button_picking() -> Pickable {
    Pickable {
        should_block_lower: true,
        is_hoverable: true,
    }
}

fn char_sel_button() -> Node {
    Node {
        height: Val::Percent(30.0),
        width: Val::Percent(20.0),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::SpaceEvenly,
        align_items: AlignItems::Center,
        ..default()
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterSelectionIcon;

#[derive(Component, Debug, Clone, Serialize, Deserialize, Default)]
#[require(Text = Text::from("NO TEXT COMPONENT"))]
pub struct CharacterSelectionText;

#[derive(Component, Debug, Clone, Copy)]
#[require(Node = lobby_subcontainer(20.0, 100.0))]
pub struct LobbySettingsSection;

fn character_selection_container(width: f32, height: f32) -> Node {
    let mut base = lobby_subcontainer(width, height);
    base.display = Display::Grid;
    base.grid_template_rows = vec![RepeatedGridTrack::auto(2)];
    base.grid_template_columns = vec![RepeatedGridTrack::auto(4)];
    base
}

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

fn make_lobby(mut commands: Commands, assets: Res<AssetServer>, game_kind: Res<CurrentGameKind>) {
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

    let char_selection = commands.spawn_empty().id();
    let mut button_map = HashMap::new();
    for character in CharacterKind::iter() {
        let sprite_path: AssetFolder = character.into();
        let handle: Handle<Image> =
            assets.load(format!("{}/{}", sprite_path.0, "lobby_portrait.png"));

        let char_string: String = character.into();
        let entity = commands
            .spawn((
                CharacterSelectionButton {
                    kind: character,
                    selected_by: vec![],
                },
                ChildOf(char_selection),
            ))
            .observe(character_selection_button_observer)
            .with_children(|p| {
                p.spawn((CharacterSelectionIcon, ImageNode::from(handle)));
                p.spawn((CharacterSelectionText, Text::from(char_string)));
            })
            .id();
        button_map.insert(character, entity);
    }

    commands.entity(char_selection).insert((
        LobbyCharacterSelection {
            buttons: button_map,
        },
        ChildOf(lobby_main),
    ));

    let settings_section = commands
        .spawn((LobbySettingsSection, ChildOf(lobby_main)))
        .id();

    let diff_section = commands
        .spawn((LobbyDifficulty, ChildOf(settings_section)))
        .id();

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

fn character_selection_button_observer(
    trigger: On<Pointer<Release>>,
    mut messages: MessageWriter<ClientChangeCharacterMessage>,
    q_button_well: Single<&LobbyCharacterSelection>,
    mut q_buttons: Query<&mut CharacterSelectionButton>,
    mut q_player: Single<(Entity, &mut PlayerInLobby), Or<(With<SinglePlayer>, With<Client>)>>,
) {
    info!("System triggered");
    if let Some(ref mut char) = q_player.1.selected_character {
        // Get the previous button selected by this person to update the selection vector
        let prev_ent = q_button_well.buttons.get(char).expect("Not Found!");
        let mut prev_button = q_buttons.get_mut(*prev_ent).unwrap();
        let pos = prev_button
            .selected_by
            .iter()
            .position(|e| *e == q_player.0)
            .unwrap();
        prev_button.selected_by.remove(pos);
    }

    if let Ok(mut b) = q_buttons.get_mut(trigger.entity) {
        q_player.1.selected_character = Some(b.kind);
        b.selected_by.push(q_player.0);
        messages.write(ClientChangeCharacterMessage { char: b.kind });
    }
}

fn mp_propagate_client_change_character_message_to_server(
    mut reader: MessageReader<ClientChangeCharacterMessage>,
    mut q_client: Single<&mut MessageSender<ClientChangeCharacterMessage>>,
) {
    for e in reader.read() {
        q_client.send::<GameMainChannel>(*e);
    }
}
