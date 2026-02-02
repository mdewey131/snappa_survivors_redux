use super::spawner::*;
use bevy::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
pub struct EnemySpawnManagerEditorPlugin;

impl Plugin for EnemySpawnManagerEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ResourceInspectorPlugin::<EnemySpawnManager>::new());
    }
}
