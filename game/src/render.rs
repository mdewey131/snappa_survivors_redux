use avian2d::prelude::Position;
#[cfg(feature = "avian_debug")]
use avian2d::prelude::*;
use bevy::{prelude::*, render::RenderSystems};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub mod animation;
pub mod camera;
pub mod enemies;
pub mod map;
pub mod menus;
pub mod player;
pub mod ui;
pub mod upgrades;
pub mod weapons;

use camera::GameMainCamera;
use enemies::SharedEnemyRenderPlugin;
use map::MapRenderPlugin;
use menus::lobby::LobbyMenuPlugin;
use player::SharedPlayerRenderPlugin;
use ui::SharedUIPlugin;
use upgrades::UpgradeRenderPlugin;

#[cfg(feature = "dev")]
use crate::shared::stats::editor::StatsEditorPlugin;

pub struct GameSharedRenderPlugin;

impl Plugin for GameSharedRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SharedUIPlugin,
            LobbyMenuPlugin,
            MapRenderPlugin,
            SharedPlayerRenderPlugin,
            SharedEnemyRenderPlugin,
            UpgradeRenderPlugin,
        ));
        #[cfg(feature = "inspector")]
        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()));
        #[cfg(feature = "avian_debug")]
        app.add_plugins(PhysicsDebugPlugin::default());
        #[cfg(feature = "dev")]
        app.add_plugins(StatsEditorPlugin);

        app.add_systems(Startup, startup).add_systems(
            PostUpdate,
            (render_y_to_z, sync_transform_to_pos)
                .chain()
                .before(RenderSystems::Prepare),
        );
    }
}

/// This marker component indicates that the entity should be treated with its z position equal to its y position.
///
/// That allows for proper sprite layering in theory
#[derive(Component, Default)]
pub struct RenderYtoZ;

fn startup(mut commands: Commands) {
    commands.spawn((Camera2d::default(), GameMainCamera::default()));
}

fn render_y_to_z(mut q_pos: Query<&mut Transform, (With<RenderYtoZ>, Changed<Transform>)>) {
    let _span = info_span!("Render Y to Z system").entered();
    for mut pos in &mut q_pos {
        // We have to rebase to the amount allowed by the 2d camera, which seems to be -1000.
        // Since that's the case, I think it will be okay to just bring this down by a few orders of magnitude
        pos.translation.z = pos.translation.y * -0.001
    }
}

/// A function that I was so sure didn't need to be written, making sure that we update an entity's transform
/// to be based on their position. This is apparently necessary?
fn sync_transform_to_pos(mut q_transform: Query<(&mut Transform, &Position), Without<ChildOf>>) {
    for (mut t, pos) in &mut q_transform {
        t.translation.x = pos.0.x;
        t.translation.y = pos.0.y;
    }
}
