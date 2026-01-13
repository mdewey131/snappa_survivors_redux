use crate::{render::RenderYtoZ, shared::enemies::*};
use avian2d::prelude::Position;
use bevy::prelude::*;
use core::marker::PhantomData;

pub struct EnemyRenderPlugin<C> {
    _mark: PhantomData<C>,
}
impl<C: Component> EnemyRenderPlugin<C> {
    pub fn new() -> Self {
        Self { _mark: PhantomData }
    }
}

impl<C: Component> Plugin for EnemyRenderPlugin<C> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                on_enemy_add::<C>, /*,
                                   (animate::<Player>, update_facing_direction::<Player>)
                                       .chain()
                                       .before(RenderSystems::ExtractCommands)*/
            ),
        );
    }
}

pub fn on_enemy_add<C: Component>(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    q_player: Query<(Entity, &Position), (Added<Enemy>, With<C>)>,
) {
    for (e, pos) in &q_player {
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
