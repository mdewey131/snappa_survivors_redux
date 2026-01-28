use std::marker::PhantomData;

use avian2d::prelude::Position;
use bevy::{ecs::query::QueryFilter, prelude::*, render::RenderSystems};
use bevy_enhanced_input::prelude::*;

use crate::{
    render::{RenderYtoZ, animation::*},
    shared::{inputs::Movement, players::Player, states::InGameState},
};

/// Handles the rendering of the player.
///
/// This is parameterized by a component because
/// a dedicated server might want to render according to
/// the replicated component, but the client only wants
/// to render on the basis of predicted
pub struct SharedPlayerRenderPlugin;

impl Plugin for SharedPlayerRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((animate::<Player>, update_player_facing_direction)
                .chain()
                .before(RenderSystems::ExtractCommands)
                .run_if(in_state(InGameState::InGame)),),
        );
    }
}

pub fn rendering_on_player_add<QF: QueryFilter>(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    q_player: Query<(Entity, &Position), (Added<Player>, QF)>,
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

pub fn update_player_facing_direction(
    mut q_animation: Query<
        (
            &mut AnimationFacing,
            &mut AnimationConfig,
            &mut Sprite,
            &Actions<Player>,
        ),
        With<Player>,
    >,
    q_movement: Query<&ActionValue, With<Action<Movement>>>,
) {
    for (mut facing, mut config, mut sprite, actions) in &mut q_animation {
        // Find the movement
        for a_ent in actions.iter() {
            if let Ok(av) = q_movement.get(a_ent) {
                let velo = av.as_axis2d();
                facing.derive_next_direction(velo);
                // TODO: Move this elsewhere
                if velo == Vec2::ZERO {
                    config.frame_timer.pause()
                } else if config.frame_timer.is_paused() {
                    config.frame_timer.unpause()
                }
                facing.update_facing(&mut config, &mut sprite)
            }
        }
    }
}
