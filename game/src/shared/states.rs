use avian2d::prelude::{Physics, PhysicsTime};
use bevy::{prelude::*, time::Stopwatch};

use crate::shared::{
    combat::CombatSystemSet,
    damage::Dead,
    players::Player,
};

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
    GameOver,
    PostGame,
}

/// This checks to see if all players are dead and, if so, begins a small timer to pause the game and move to game over.
/// Its done this way just to make really sure that players are actually dead
#[derive(Resource, Default)]
pub struct GameOverTimer {
    pub timer: Option<Timer>,
}
impl GameOverTimer {
    pub fn start_timer(&mut self) {
        self.timer = Some(Timer::from_seconds(0.3, TimerMode::Once))
    }
}

pub fn add_game_over_timer(mut commands: Commands) {
    commands.insert_resource(GameOverTimer::default());
}

pub fn check_game_over(
    // Not virtual time because we don't want you pausing to delay the inevitable here. Just get it over with
    time: Res<Time>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<InGameState>>,
    mut res: ResMut<GameOverTimer>,
    q_players: Query<(), (With<Player>, Without<Dead>)>,
) {
    if q_players.is_empty() && res.timer.is_none()  {
        res.start_timer();
    } else if q_players.is_empty() {
        let timer = res.timer.as_mut().unwrap();
        timer.tick(time.delta());
        if timer.is_finished() {
            app_state.set(AppState::GameOver);
            game_state.set(InGameState::OutOfGame);
        }
    } else if !q_players.is_empty() && res.timer.is_some() {
        res.timer = None;
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct InGameTime(pub Stopwatch);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct InGamePauseManager {
    paused_from: InGameState,
}

pub struct SharedStatesPlugin;
impl Plugin for SharedStatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>().init_state::<InGameState>();
        app.add_systems(
            OnEnter(AppState::InGame),
            (spawn_game_timer, set_game_state_in_game),
        )
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

pub fn pause_in_game_state(
    mut commands: Commands,
    c_state: Res<State<InGameState>>,
    mut next_state: ResMut<NextState<InGameState>>,
) {
    let pause_state = InGamePauseManager {
        paused_from: *(c_state.get()),
    };

    commands.insert_resource(pause_state);

    next_state.set(InGameState::Paused);
}

pub fn unpause_in_game_state(
    pause: Res<InGamePauseManager>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<InGameState>>,
) {
    let next = pause.paused_from;

    commands.remove_resource::<InGamePauseManager>();
    next_state.set(next);
}

pub fn set_app_state_in_game(mut app_state: ResMut<NextState<AppState>>) {
    app_state.set(AppState::InGame);
}
/// This follows after the app state to prevent race conditions from one of these being enabled before the other
pub fn set_game_state_in_game(mut game_state: ResMut<NextState<InGameState>>) {
    game_state.set(InGameState::InGame);
}
