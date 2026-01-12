use std::marker::PhantomData;

use avian2d::prelude::Position;
use bevy::{prelude::*, render::RenderSystems};

use crate::{
    render::{RenderYtoZ, animation::*},
    shared::players::Player,
};

/// Handles the rendering of the player.
///
/// This is parameterized by a component because
/// a dedicated server might want to render according to
/// the replicated component, but the client only wants
/// to render on the basis of predicted
pub struct PlayerRenderPlugin<C> {
    _mark: PhantomData<C>,
}
impl<C: Component> PlayerRenderPlugin<C> {
    pub fn new() -> Self {
        Self { _mark: PhantomData }
    }
}

impl<C: Component> Plugin for PlayerRenderPlugin<C> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                on_player_add::<C>,
                (animate::<Player>, update_facing_direction::<Player>)
                    .chain()
                    .before(RenderSystems::ExtractCommands),
            ),
        );
    }
}

pub fn on_player_add<C: Component>(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    q_player: Query<(Entity, &Position), (Added<Player>, With<C>)>,
) {
    for (e, pos) in &q_player {
        let handle: Handle<Image> = assets.load("survivors/dewey/sprite_2-Sheet.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 4, 4, None, None);
        let tex_atlas = layouts.add(layout);
        let animation = AnimationConfig::new(0, 3, 4);

        let facing = AnimationFacing {
            tex_width: 4,
            ..default()
        };
        commands.entity(e).insert((
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
            Transform::from_translation(pos.0.extend(pos.0.y)),
            RenderYtoZ,
        ));
    }
}
