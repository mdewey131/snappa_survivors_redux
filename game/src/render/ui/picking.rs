pub use bevy::prelude::*;

#[derive(Component, Debug, Clone, Default, Reflect)]
pub struct SelectedBy(pub Vec<Entity>);
