//! Responsible for handling render elmeents related to the game's map
//!
//!
use bevy::prelude::*;

use crate::shared::{game_rules::GameRules, states::AppState};

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
        app.add_systems(OnEnter(AppState::LoadingLevel), load_map_chunks);
    }
}

fn load_map_chunks(mut commands: Commands, _rules: Res<GameRules>, assets: Res<AssetServer>) {
    // Making the map somewhat huge to start
    let tiles = Vec2::new(64.0, 64.0);
    let texture_size = Vec2::new(128.0, 128.0);
    let map = commands.spawn(MapBackground).id();

    let (total_size_x, total_size_y) = (tiles.x * texture_size.x, tiles.y * texture_size.y);
    for x in (0..tiles.x as u32) {
        for y in (0..tiles.y as u32) {
            let texture: Handle<Image> = assets.load("maps/grass_bg.png");
            commands.spawn((
                Sprite::from(texture),
                Transform::from_translation(Vec3::new(
                    x as f32 * texture_size.x - (total_size_x / 2.0),
                    y as f32 * texture_size.y - (total_size_y / 2.0),
                    -1000.0,
                )),
                ChunkOf(map),
            ));
        }
    }
}
