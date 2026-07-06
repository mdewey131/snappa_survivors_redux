//! Responsible for handling render elmeents related to the game's map
//!
//!
use bevy::prelude::*;

use crate::shared::{
    game_rules::GameRules,
    loading::{LevelLoadingState, track_loading_asset},
    states::AppState,
};

/// The map's rendered elements will work off of chunks so that I can spawn and despawn things somewhat easily.
/// I'm anticipating some eventual first party support for this kind of thing, so the goal here is to keep this
/// implementation simple
#[derive(Component, Debug, Clone, Copy)]
pub struct MapChunk;

/// Requires Visibility to tamp down on a noisy warning
#[derive(Component)]
#[require(Visibility = Visibility::Visible)]
pub struct MapBackground;

#[derive(Component)]
#[relationship(relationship_target = HasChunks)]
pub struct ChunkOf(pub Entity);

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
            add_tile_sprites.run_if(in_state(AppState::LoadingLevel)),
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
    _game_rules: Res<GameRules>,
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

fn add_tile_sprites(
    mut commands: Commands,
    map_asset: Res<MapAssets>,
    q_tiles: Query<Entity, (With<MapChunk>, Without<Sprite>)>,
) {
    for tile in q_tiles {
        let texture: Handle<Image> = map_asset.tilemap.as_ref().unwrap().clone();
        commands.entity(tile).insert(Sprite::from(texture));
    }
}
