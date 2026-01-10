use bevy::{ecs::system::SystemId, prelude::*};
use lightyear::prelude::*;

use crate::shared::{
    GameMainChannel,
    game_rules::GameRules,
    lobby::{ClientStartGameMessage, ServerStartLoadingGameMessage},
    states::{AppState, InGameState},
};

pub struct ClientGameLoadingPlugin;

impl Plugin for ClientGameLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                tmp_send_game_start_message.run_if(|button: Res<ButtonInput<KeyCode>>| {
                    button.just_pressed(KeyCode::Enter)
                }),
                client_on_receive_start_loading_message,
            )
                .run_if(in_state(AppState::Lobby)),
        )
        .add_systems(OnEnter(AppState::LoadingLevel), tmp_move_to_game);
    }
}

pub fn client_transition_to_loading_state(
    arg_in: In<GameRules>,
    mut rules: ResMut<GameRules>,
    mut state: ResMut<NextState<AppState>>,
) {
    *rules = *arg_in;
    state.set(AppState::LoadingLevel)
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

fn tmp_send_game_start_message(mut q_sender: Query<&mut MessageSender<ClientStartGameMessage>>) {
    for mut sender in &mut q_sender {
        sender.send::<GameMainChannel>(ClientStartGameMessage)
    }
}

/// For now, loading does nothing because I don't want to figure it out. Let's just get to the game stuff
fn tmp_move_to_game(
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<InGameState>>,
) {
    app_state.set(AppState::InGame);
    game_state.set(InGameState::InGame);
}
