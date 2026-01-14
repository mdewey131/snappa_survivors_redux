//! Responsible for managing the game that is going to happen.
//!
//! This component gets called when we're about to start a game and allows players
//! to customize it in order to set up the game state that you want to load
//!
//! The goal of doing it this way is to delegate things related to loading a game
//! to one central source of truth, and to have that be a source that can be modified

use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shared::{GameMainChannel, states::AppState};

pub struct SharedGameRulesPlugin;

impl Plugin for SharedGameRulesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Lobby), add_game_rules_resource);
        app.register_message::<ChangeGameRuleMessage<MapKind>>()
            .add_direction(NetworkDirection::ClientToServer);

        app.register_message::<ChangeGameRuleMessage<Difficulty>>()
            .add_direction(NetworkDirection::ClientToServer);
    }
}

pub fn add_game_rules_resource(mut commands: Commands) {
    commands.insert_resource(GameRules::default())
}

/// The central component for how a game gets set up.
/// My idea at this point is that the GameRules object
/// is spawned as a replicated component by the server at the start of a lobby.
/// upon the transition to the loading state, this component
/// calls for each of its different elements to be loaded, which the
/// client will also do.
/// Once all clients have loaded, the server can move this into
#[derive(Resource, Default, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct GameRules {
    pub map_type: MapKind,
    pub difficulty: Difficulty,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Reflect)]
pub enum MapKind {
    #[default]
    TheGreens,
}
impl GameRuleField for MapKind {
    fn set_field(&self, rules: &mut GameRules) {
        rules.map_type = *self
    }
}

unsafe impl Send for MapKind {}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Reflect)]
pub enum Difficulty {
    #[default]
    Easy,
    Normal,
    Hard,
}
unsafe impl Send for Difficulty {}
impl GameRuleField for Difficulty {
    fn set_field(&self, rules: &mut GameRules) {
        rules.difficulty = *self
    }
}

/*
trait ChangesGameRules: lightyear::prelude::Message {
    fn apply(&self, rules: &mut GameRules);
}
*/
pub trait GameRuleField: lightyear::prelude::Message + Copy {
    fn set_field(&self, rules: &mut GameRules);
}

#[derive(Message, Clone, Copy, Debug, Serialize, Deserialize)]
/// Sent from the client to the server in the lobby
pub struct ChangeGameRuleMessage<F: GameRuleField> {
    to: F,
}

/// It's unlikely that this will really need to resolve the issues
/// related to the fact that there could be multiple receivers. But
/// that's a good thing to check
pub fn receive_game_change_message<F: GameRuleField>(
    mut rules: ResMut<GameRules>,
    mut q_receiver: Query<&mut MessageReceiver<ChangeGameRuleMessage<F>>>,
) {
    for mut rec in &mut q_receiver {
        for mess in rec.receive() {
            mess.to.set_field(rules.as_mut());
        }
    }
}

/// A bevy system run by the client, with the proper field value set in
///
/// Generally, the game will run these things on the basis of UI widgets
/// firing the logic to send it
pub fn send_game_change_message_callback<F: GameRuleField>(
    state_in: In<F>,
    mut local_rules: ResMut<GameRules>,
    mut q_sender: Option<Single<&mut MessageSender<ChangeGameRuleMessage<F>>>>,
) {
    state_in.0.set_field(local_rules.as_mut());
    let mess = ChangeGameRuleMessage { to: state_in.0 };
    if let Some(ref mut sender) = q_sender {
        sender.send::<GameMainChannel>(mess);
    }
}
