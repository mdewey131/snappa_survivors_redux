use bevy::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum GameKinds {
    SinglePlayer,
    MultiPlayer,
}

/// The marker component and types that is used to differentiate between
/// We will have lightyear do the work of making predicted and replicated
#[derive(Component, Debug, Clone, Copy)]
pub struct SinglePlayer;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CurrentGameKind(pub Option<GameKinds>);

pub struct GameKindsPlugin;

impl Plugin for GameKindsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentGameKind>();
    }
}
