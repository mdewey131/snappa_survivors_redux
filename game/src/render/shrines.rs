use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{render::RenderYtoZ, shared::interactables::BeerShrine};

pub struct SharedShrinesRenderPlugin;

impl Plugin for SharedShrinesRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, rendering_on_shrine_add);
    }
}

fn rendering_on_shrine_add(
    mut commands: Commands,
    asset: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    q_shrine: Query<(Entity, &Position), (Added<BeerShrine>, Without<Sprite>)>,
) {
    for (shrine, pos) in &q_shrine {
        let handle: Handle<Image> = asset.load("shrines/beer_shrine-Sheet.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(96), 8, 1, None, None);
        let tex_atlas = layouts.add(layout);

        commands.entity(shrine).insert((
            Sprite {
                image: handle.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: tex_atlas.clone(),
                    index: 0,
                }),
                ..default()
            },
            Transform::from_translation(pos.0.extend(pos.0.y)),
            RenderYtoZ::default(),
        ));
    }
}
