use std::marker::PhantomData;

use avian2d::prelude::Position;
use bevy::prelude::*;

use crate::{render::RenderYtoZ, shared::players::Player};

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
        app.add_systems(Update, on_player_add::<C>);
    }
}

pub fn on_player_add<C: Component>(
    mut commands: Commands,
    assets: Res<AssetServer>,
    q_player: Query<(Entity, &Position), (Added<Player>, With<C>)>,
) {
    for (e, pos) in &q_player {
        let handle: Handle<Image> = assets.load("survivors/dewey/sprite.png");
        commands.entity(e).insert((
            Sprite::from_image(handle),
            Transform::from_translation(pos.0.extend(pos.0.y)),
            RenderYtoZ,
        ));
    }
}
