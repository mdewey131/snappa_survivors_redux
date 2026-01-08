use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub struct GameSharedRenderPlugin;

impl Plugin for GameSharedRenderPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "inspector")]
        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()));
        app.add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands) {
    commands.spawn((Camera2d::default()));
}
