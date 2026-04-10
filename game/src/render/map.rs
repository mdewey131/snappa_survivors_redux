//! Responsible for handling render elmeents related to the game's map
//!
//!
use bevy::prelude::*;

use crate::{
    render::MapAssets,
    shared::{
        game_rules::GameRules,
        loading::{LevelLoadingState, track_loading_asset},
        states::AppState,
    },
};

/// The map's rendered elements will work off of chunks so that I can spawn and despawn things somewhat easily.
/// I'm anticipating some eventual first party support for this kind of thing, so the goal here is to keep this
/// implementation simple
#[derive(Component, Debug, Clone, Copy)]
pub struct MapChunk;

#[derive(Component)]
pub struct MapBackground;

#[derive(Component)]
#[relationship(relationship_target = HasChunks)]
pub struct ChunkOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = ChunkOf)]
pub struct HasChunks(Vec<Entity>);

pub struct MapRenderPlugin;

impl Plugin for MapRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(LevelLoadingState::LevelLoading),
            load_map_tile.pipe(track_loading_asset),
        )
        .add_systems(
            OnEnter(LevelLoadingState::LevelReady),
            assemble_map_chunks.run_if(in_state(AppState::LoadingLevel)),
        );
    }
}
#[derive(Resource, Debug, Default)]
pub struct MapAssets {
    // Just an image right now because I haven't made a tilemap yet
    pub tilemap: Option<Handle<Image>>,
}

fn load_map_tile(
    mut commands: Commands,
    game_rules: Res<GameRules>,
    assets: Res<AssetServer>,
) -> Vec<UntypedHandle> {
    let handle: Handle<Image> = assets.load("maps/grass_bg.png");
    let ret = handle.clone();
    let asset = MapAssets {
        tilemap: Some(handle),
    };
    commands.insert_resource(asset);

    vec![ret.untyped()]
}

fn assemble_map_chunks(mut commands: Commands, map_asset: Res<MapAssets>) {
    // Making the map somewhat huge to start
    let tiles = Vec2::new(64.0, 64.0);
    let texture_size = Vec2::new(128.0, 128.0);
    let map = commands
        .spawn((
            MapBackground,
            Transform::default(),
            Visibility::Visible,
            DespawnOnExit(AppState::InGame),
        ))
        .id();

    let (total_size_x, total_size_y) = (tiles.x * texture_size.x, tiles.y * texture_size.y);
    for x in (0..tiles.x as u32) {
        for y in (0..tiles.y as u32) {
            let texture: Handle<Image> = map_asset.tilemap.as_ref().unwrap().clone();
            commands.spawn((
                Sprite::from(texture),
                Transform::from_translation(Vec3::new(
                    x as f32 * texture_size.x - (total_size_x / 2.0),
                    y as f32 * texture_size.y - (total_size_y / 2.0),
                    -1000.0,
                )),
                ChunkOf(map),
                // this is for bevy inspector egui reasons
                ChildOf(map),
            ));
        }
    }
}
