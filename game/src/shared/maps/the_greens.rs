use crate::{
    render::{
        RenderYtoZ,
        animation::AnimationConfig,
        map::{ChunkOf, MapBackground, MapChunk},
    },
    shared::{game_kinds::CurrentGameKind, lobby::PlayerInLobby, players::*, states::AppState},
    utils::SpawnPattern,
};
use bevy::prelude::*;
use bluenoise::BlueNoise;
use lightyear::prelude::*;
use rand::rngs::{SmallRng, ThreadRng};

pub const THE_GREENS_NUM_TILES: (u32, u32) = (100, 100);
pub const THE_GREENS_TILE_SIZE: (f32, f32) = (128.0, 128.0);
pub const THE_GREENS_MIN_WIDTH_BETWEEN_INTERACTABLES: f32 = 50.0;

pub fn spawn_characters_the_greens(
    mut commands: Commands,
    game_kinds: Res<CurrentGameKind>,
    q_player: Query<(Entity, &PlayerInLobby, Option<&RemoteId>)>,
) {
    let n_char = q_player.iter().len();
    let mut spawn_pos = SpawnPattern::Circle {
        amount: n_char as u8,
        center: Vec2::ZERO,
        radius: 500.0,
        radius_only: true,
    }
    .to_positions();

    spawn_characters(&mut spawn_pos, &mut commands, &game_kinds, &q_player);
}

/// Don't do it like this later
pub fn spawn_interactables_the_greens(
    mut commands: Commands,
    asset: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let total_size_x = (THE_GREENS_NUM_TILES.0 as f32) * THE_GREENS_TILE_SIZE.0;
    let total_size_y = (THE_GREENS_NUM_TILES.1 as f32) * THE_GREENS_TILE_SIZE.1;
    let mut noise = BlueNoise::<SmallRng>::new(
        total_size_x,
        total_size_y,
        THE_GREENS_MIN_WIDTH_BETWEEN_INTERACTABLES,
    );
    let noise = noise.with_samples(500).with_seed(10);
    let handle: Handle<Image> = asset.load("shrines/beer_shrine-Sheet.png");

    let layout = TextureAtlasLayout::from_grid(UVec2::splat(96), 1, 8, None, None);
    let tex_atlas = layouts.add(layout);
    let animation = AnimationConfig::new(0, 7, 8);

    for point in noise.take(500) {
        commands.spawn((
            Sprite {
                image: handle.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: tex_atlas.clone(),
                    index: 0,
                }),
                ..default()
            },
            animation.clone(),
            Transform::from_translation(Vec3::new(
                point.x - (total_size_x / 2.0),
                point.y - (total_size_x / 2.0),
                point.y - (total_size_x / 2.0),
            )),
            RenderYtoZ::default(),
        ));
    }
}

pub fn enemy_spawners_the_greens() {}

pub fn map_elements_the_greens() {}

pub fn map_chunks_the_greens(mut commands: Commands) {
    // Making the map somewhat huge to start
    let texture_size = Vec2::new(128.0, 128.0);
    let map = commands
        .spawn((
            MapBackground,
            Transform::default(),
            DespawnOnExit(AppState::InGame),
        ))
        .id();

    let total_size_x = (THE_GREENS_NUM_TILES.0 as f32) * THE_GREENS_TILE_SIZE.0;
    let total_size_y = (THE_GREENS_NUM_TILES.1 as f32) * THE_GREENS_TILE_SIZE.1;
    for x in (0..THE_GREENS_NUM_TILES.0) {
        for y in (0..THE_GREENS_NUM_TILES.1) {
            commands.spawn((
                Transform::from_translation(Vec3::new(
                    x as f32 * texture_size.x - (total_size_x / 2.0),
                    y as f32 * texture_size.y - (total_size_y / 2.0),
                    -1000.0,
                )),
                ChunkOf(map),
                MapChunk,
                // this is for bevy inspector egui reasons
                ChildOf(map),
            ));
        }
    }
}
