use crate::{render::RenderYtoZ, shared::enemies::*};
use avian2d::prelude::Position;
use bevy::{ecs::query::QueryFilter, prelude::*};
use core::marker::PhantomData;

pub struct SharedEnemyRenderPlugin;

impl Plugin for SharedEnemyRenderPlugin {
    fn build(&self, app: &mut App) {
        /*
        app.add_systems(
            Update,
            (
                                    (animate::<Player>, update_facing_direction::<Player>)
                                        .chain()
                                        .before(RenderSystems::ExtractCommands)
            ),

        );
        */
    }
}

pub fn rendering_on_enemy_add<QF: QueryFilter>(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    q_enemy: Query<(Entity, &Position), (Added<Enemy>, QF)>,
) {
    for (e, pos) in &q_enemy {
        let handle: Handle<Image> = assets.load("enemies/faceless/sprite.png");
        //let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 4, 4, None, None);
        //let tex_atlas = layouts.add(layout);
        /*
        let animation = AnimationConfig::new(0, 3, 4);

        let facing = AnimationFacing {
            tex_width: 4,
            ..default()
        };
        */
        commands.entity(e).insert((
            Sprite::from_image(handle),
            /*
            Sprite {
                image: handle,
                texture_atlas: Some(TextureAtlas {
                    layout: tex_atlas.clone(),
                    index: 0,
                }),
                ..default()
            },
            facing,
            animation,
            */
            Transform::from_translation(pos.0.extend(pos.0.y)),
            RenderYtoZ,
        ));
    }
}
