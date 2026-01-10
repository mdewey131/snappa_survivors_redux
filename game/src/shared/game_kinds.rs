use bevy::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum GameKinds {
    SinglePlayer,
    MultiPlayer,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CurrentGameKind(pub Option<GameKinds>);

pub struct GameKindsPlugin;

impl Plugin for GameKindsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentGameKind>();
    }
}
