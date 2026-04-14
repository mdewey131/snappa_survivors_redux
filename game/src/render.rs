use crate::{
    render::menus::loading_screen::LoadingScreenPlugin,
    shared::{
        enemies::EnemyKind,
        game_rules::GameRules,
        loading::{LevelLoadingState, LoadingAssets},
        players::CharacterKind,
        states::AppState,
        weapons::WeaponKind,
    },
};
use avian2d::prelude::Position;
#[cfg(feature = "avian_debug")]
use avian2d::prelude::*;
use bevy::{
    asset::UntypedAssetId, platform::collections::HashMap, prelude::*, render::RenderSystems,
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub mod animation;
pub mod camera;
pub mod enemies;
pub mod hud;
pub mod map;
pub mod menus;
pub mod pickups;
pub mod player;
pub mod ui;
pub mod upgrades;
pub mod weapons;

use camera::*;
use enemies::SharedEnemyRenderPlugin;
use map::MapRenderPlugin;
use menus::lobby::LobbyMenuPlugin;
use pickups::*;
use player::SharedPlayerRenderPlugin;
use ui::SharedUIPlugin;
use upgrades::UpgradeRenderPlugin;

use crate::shared::loading::track_loading_asset;
#[cfg(feature = "dev")]
use crate::shared::{
    enemies::editor::EnemySpawnManagerEditorPlugin, stats::editor::StatsEditorPlugin,
};

pub struct GameSharedRenderPlugin;

impl Plugin for GameSharedRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SharedUIPlugin,
            LobbyMenuPlugin,
            LoadingScreenPlugin,
            MapRenderPlugin,
            SharedPlayerRenderPlugin,
            SharedEnemyRenderPlugin,
            SharedPickupsRenderPlugin,
            UpgradeRenderPlugin,
        ));
        #[cfg(feature = "inspector")]
        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()));
        #[cfg(feature = "avian_debug")]
        app.add_plugins(PhysicsDebugPlugin::default());
        #[cfg(feature = "dev")]
        app.add_plugins(StatsEditorPlugin);
        #[cfg(feature = "dev")]
        app.add_plugins(EnemySpawnManagerEditorPlugin);

        app.add_systems(Startup, startup)
            .add_systems(
                PostUpdate,
                (render_y_to_z, sync_transform_to_pos)
                    .chain()
                    .before(RenderSystems::Prepare),
            )
            .add_systems(
                Update,
                update_free_cam_position.run_if(in_state(AppState::InGame)),
            );
    }
}

/// This resource tracks the assets that the game needs
/// while it is currently engaged in a level.
/// This resource can be used to prevent loading assets during gameplay,
/// and delegating this to the app's loading state instead
#[derive(Resource, Debug, Default)]
pub struct LevelRenderAssets {
    /// TODO: Make this more than one image
    ///
    /// Optional because I want Default
    pub map_tiles: Option<Handle<Image>>,
    pub projectiles: HashMap<String, Handle<Image>>,
    pub weapons: HashMap<WeaponKind, Handle<Image>>,
    pub enemies: HashMap<EnemyKind, Handle<Image>>,
    pub characters: HashMap<CharacterKind, Handle<Image>>,
}

/// This component indicates that the entity should be treated with its z position equal to its y position.
///
/// That allows for proper sprite layering in theory,
/// with the allotment for some things to offset themselves in cases where they need to be drawn ahead/behind of nearby things
/// (e.g. when trying to show something in the air)
#[derive(Component, Default)]
pub struct RenderYtoZ {
    offset: f32,
}

impl RenderYtoZ {
    fn new(offset: f32) -> Self {
        Self { offset }
    }
}

fn startup(mut commands: Commands) {
    commands.spawn((Camera2d::default(), GameMainCamera::default()));
}

fn render_y_to_z(mut q_pos: Query<(&mut Transform, &RenderYtoZ), Changed<Transform>>) {
    let _span = info_span!("Render Y to Z system").entered();
    for (mut pos, render) in &mut q_pos {
        // We have to rebase to the amount allowed by the 2d camera, which seems to be -1000.
        // Since that's the case, I think it will be okay to just bring this down by a few orders of magnitude
        let new_z = pos.translation.y * -0.001 + render.offset;
        pos.translation.z = new_z;
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
