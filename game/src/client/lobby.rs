use bevy::{ecs::system::SystemId, prelude::*};
use lightyear::prelude::*;

use crate::{
    client::load_game::client_transition_to_loading_state,
    render::{menus::lobby::*, ui::button::*},
    shared::{
        GameMainChannel,
        game_kinds::{is_single_player, *},
        game_rules::GameRules,
        lobby::{ClientStartGameMessage, ServerStartLoadingGameMessage},
        states::AppState,
    },
};
pub struct ClientGameLobbyPlugin;

impl Plugin for ClientGameLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (
                    tmp_send_network_game_start_message.run_if(not(is_single_player)),
                    tmp_send_1p_game_start_message.run_if(is_single_player),
                )
                    .run_if(enter_pressed),
                client_on_receive_start_loading_message.run_if(not(is_single_player)),
                client_on_receive_start_game_message.run_if(is_single_player),
            )
                .run_if(in_state(AppState::Lobby)),
        )
        .add_observer(spawn_lobby_back_button)
        .add_observer(observe_lobby_back_button);
    }
}

/// Most of our loading elements should be thought of as "server driven".
/// Otherwise, you're going to end up in a position where you double load things
/// on the client in the single player mode. But, if the responsibility for
/// loading things always falls on the server, then the client's responsibilty
/// is to always follow what the server is doing
pub fn client_on_receive_start_loading_message(
    mut commands: Commands,
    mut system_to_run: Local<Option<SystemId<In<GameRules>, ()>>>,
    mut q_rec: Query<&mut MessageReceiver<ServerStartLoadingGameMessage>>,
) {
    let mut run_with = None;
    for mut rec in &mut q_rec {
        for message in rec.receive() {
            run_with = Some(message);
            break;
        }
    }

    if let Some(m) = run_with {
        if system_to_run.is_none() {
            *system_to_run = Some(commands.register_system(client_transition_to_loading_state));
        }
        commands.run_system_with(system_to_run.unwrap(), m.rules)
    }
}

/// In single player, this is responsible for getting us started on the loading state
pub fn client_on_receive_start_game_message(
    mut commands: Commands,
    mut system_to_run: Local<Option<SystemId<In<GameRules>, ()>>>,
    game_rules: Res<GameRules>,
    mut messages: MessageReader<ClientStartGameMessage>,
) {
    let mut run_with = None;
    for m in messages.read() {
        run_with = Some(game_rules);
        break;
    }

    if let Some(rules) = run_with {
        if system_to_run.is_none() {
            *system_to_run = Some(commands.register_system(client_transition_to_loading_state));
        }
        commands.run_system_with(system_to_run.unwrap(), *rules)
    }
}

fn tmp_send_network_game_start_message(
    mut q_sender: Query<&mut MessageSender<ClientStartGameMessage>>,
) {
    info!("Sending!");
    for mut sender in &mut q_sender {
        sender.send::<GameMainChannel>(ClientStartGameMessage)
    }
}

fn tmp_send_1p_game_start_message(mut messages: MessageWriter<ClientStartGameMessage>) {
    messages.write(ClientStartGameMessage);
}

fn enter_pressed(button: Res<ButtonInput<KeyCode>>) -> bool {
    button.just_pressed(KeyCode::Enter)
}

fn observe_lobby_back_button(
    trigger: On<ButtonReleased>,
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    game_kind: Res<CurrentGameKind>,
    q_client: Single<Entity, With<Client>>,
    q_back_button: Query<(), With<LobbyBackButton>>,
) {
    if let Ok(()) = q_back_button.get(trigger.entity) {
        commands.trigger(Disconnect { entity: *q_client });
        let next_state = match game_kind.0.unwrap() {
            GameKinds::MultiPlayer => AppState::MultiplayerServerSelection,
            GameKinds::SinglePlayer => AppState::MainMenu,
        };
        state.set(next_state)
    }
}
