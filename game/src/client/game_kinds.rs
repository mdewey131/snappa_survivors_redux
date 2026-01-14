use crate::shared::{
    enemies::Enemy,
    game_kinds::{CurrentGameKind, GameKinds, SinglePlayer},
    players::Player,
    states::AppState,
};
use bevy::prelude::*;
use core::marker::PhantomData;

pub struct ClientGameKindsPlugin;

impl Plugin for ClientGameKindsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AddSinglePlayerComponentPlugin::<Player>::new(),
            AddSinglePlayerComponentPlugin::<Enemy>::new(),
        ));
    }
}

pub struct AddSinglePlayerComponentPlugin<C> {
    _mark: PhantomData<C>,
}
impl<C: Component> AddSinglePlayerComponentPlugin<C> {
    pub fn new() -> Self {
        Self { _mark: PhantomData }
    }
}

impl<C: Component> Plugin for AddSinglePlayerComponentPlugin<C> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LoadingLevel),
            add_single_player_observer::<C>.run_if(is_single_player),
        );
    }
}

#[derive(Component)]
pub struct SinglePlayerObserver<C: Component> {
    _mark: PhantomData<C>,
}

fn add_single_player_observer<C: Component>(mut commands: Commands) {
    commands.spawn((
        SinglePlayerObserver { _mark: PhantomData },
        Observer::new(attach_singleplayer_component::<C>),
    ));
}

fn attach_singleplayer_component<C: Component>(t: On<Add, C>, mut commands: Commands) {
    commands.entity(t.entity).insert(SinglePlayer);
}
