use crate::shared::{
    enemies::Enemy,
    game_kinds::{CurrentGameKind, GameKinds, is_single_player},
    players::Player,
    projectiles::Projectile,
    states::AppState,
};
use bevy::prelude::*;
use core::marker::PhantomData;
use lightyear::prelude::*;

pub struct DedicatedServerGameKindsPlugin;

impl Plugin for DedicatedServerGameKindsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AddReplicationComponentsPlugin::<Player>::new(),
            AddReplicationComponentsPlugin::<Enemy>::new(),
            AddReplicationComponentsPlugin::<Projectile>::new(),
        ));
    }
}

pub struct AddReplicationComponentsPlugin<C> {
    _mark: PhantomData<C>,
}
impl<C: Component> AddReplicationComponentsPlugin<C> {
    pub fn new() -> Self {
        Self { _mark: PhantomData }
    }
}

impl<C: Component> Plugin for AddReplicationComponentsPlugin<C> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LoadingLevel),
            add_replication_observer::<C>.run_if(not(is_single_player)),
        );
    }
}

#[derive(Component)]
pub struct MultiPlayerObserver<C: Component> {
    _mark: PhantomData<C>,
}

fn add_replication_observer<C: Component>(mut commands: Commands) {
    info!("Spawning observer");
    commands.spawn((
        MultiPlayerObserver {
            _mark: PhantomData::<C>::default(),
        },
        Observer::new(attach_mp_component::<C>),
    ));
}

fn attach_mp_component<C: Component>(t: On<Add, C>, mut commands: Commands) {
    commands.entity(t.entity).insert((
        Replicate::to_clients(NetworkTarget::All),
        PredictionTarget::to_clients(NetworkTarget::All),
    ));
}
