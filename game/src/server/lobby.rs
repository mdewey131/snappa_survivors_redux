use crate::shared::{
    GameMainChannel, game_rules::*, lobby::*, states::AppState,
};
use bevy::prelude::*;
use lightyear::prelude::*;

pub struct DedicatedServerLobbyPlugin;
impl Plugin for DedicatedServerLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Lobby), server_spawn_lobby)
            .add_systems(
                Update,
                (
                    server_on_receive_start_game_message.run_if(in_state(AppState::Lobby)),
                    server_on_receive_character_change_message.run_if(in_state(AppState::Lobby)),
                ),
            );
    }
}

fn server_spawn_lobby(mut commands: Commands) {
    commands.spawn(
        Lobby {
            players: vec![],
            max_players: 8,
        } ,
    );
}

pub fn server_move_to_loading_state(state: &mut ResMut<NextState<AppState>>) {
    state.set(AppState::LoadingLevel)
}

/// This system is important, but does some weird stuff depending on the context.
///
/// In the single player, this server move to the loading state should change over the app
/// state and kick off the loading process BEFORE the client systems are able to do the same
/// thing.
///
/// In a dedicated server environment, this just moves the server on to the loading state, and
/// the client functions are able to move themselves with the systems that they have
pub fn server_on_receive_start_game_message(
    rules: Res<GameRules>,
    mut state: ResMut<NextState<AppState>>,
    mut q_receiver: Query<&mut MessageReceiver<ClientStartGameMessage>>,
    mut q_sender: Query<&mut MessageSender<ServerStartLoadingGameMessage>>,
) {
    let mut run = false;
    for mut rec in &mut q_receiver {
        for _m in rec.receive() {
            run = true;
            break;
        }
    }
    if run {
        for mut send in &mut q_sender {
            send.send::<GameMainChannel>(ServerStartLoadingGameMessage { rules: *rules });
            server_move_to_loading_state(&mut state);
        }
    }
}

fn server_on_receive_character_change_message(
    mut q_receiver: Query<(
        &LocalId,
        &mut MessageReceiver<ClientChangeCharacterMessage>,
        &mut PlayerInLobby,
    )>,
) {
    for (_local, mut rec, mut player) in &mut q_receiver {
        for message in rec.receive() {
            player.selected_character = Some(message.char);
        }
    }
}
