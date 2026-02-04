use avian2d::prelude::{Physics, PhysicsTime};
use bevy::{prelude::*, time::Stopwatch, ui::FixedMeasure};

use crate::shared::combat::CombatSystemSet;

/// Handles all of the logic that is relevant to the game loop.
#[derive(States, Component, Clone, PartialEq, Eq, Hash, Debug, Default, Copy)]
pub enum InGameState {
    #[default]
    OutOfGame,
    InGame,
    SelectingUpgrades,
    /// The game is not running, but it's not because we're selecting upgrades.
    Paused,
}

/// The different states of the app on the server and the client.
#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default, Copy)]
#[states(scoped_entities)]
pub enum AppState {
    #[default]
    AppInit,
    GameSplash,
    MainMenu,
    MultiplayerServerSelection,
    EstablishServerConnection,
    Lobby,
    LoadingLevel,
    InGame,
    PostGame,
}

/// Provides a state to open the pause menu. This is used so as to be
/// orthogonal to the in game state's idea of pausing, because
/// "it's an online game, you can't pause it" needs to be true on the client
#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum PauseState {
    #[default]
    Unpaused,
    Paused,
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct InGameTime(pub Stopwatch);

pub struct SharedStatesPlugin;
impl Plugin for SharedStatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>().init_state::<InGameState>();
        app.add_systems(OnEnter(AppState::InGame), spawn_game_timer)
            .add_systems(
                FixedUpdate,
                tick_in_game_time
                    .run_if(in_state(InGameState::InGame))
                    .in_set(CombatSystemSet::PreCombat),
            );
        app.add_systems(OnExit(InGameState::InGame), pause_combat);
        app.add_systems(OnEnter(InGameState::InGame), resume_combat);
    }
}

fn pause_combat(mut physics: ResMut<Time<Physics>>, mut game_timer: ResMut<InGameTime>) {
    physics.pause();
    game_timer.pause();
}

fn resume_combat(mut physics: ResMut<Time<Physics>>, mut game_timer: ResMut<InGameTime>) {
    physics.unpause();
    game_timer.unpause();
}

fn spawn_game_timer(mut commands: Commands) {
    commands.insert_resource(InGameTime(Stopwatch::new()));
}
fn tick_in_game_time(time: Res<Time<Virtual>>, mut timer: ResMut<InGameTime>) {
    timer.tick(time.delta());
}
