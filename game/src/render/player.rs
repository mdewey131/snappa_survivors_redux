use std::marker::PhantomData;

use avian2d::prelude::{LinearVelocity, Position};
use bevy::{prelude::*, render::RenderSystems};
use bevy_enhanced_input::prelude::*;

use crate::{
    render::{RenderYtoZ, animation::*},
    shared::{
        combat::CharacterFacing,
        interactables::Interactable,
        players::Player,
        states::{AppState, InGameState},
        weapons::find_closest_in_list,
    },
};

pub struct SharedPlayerRenderPlugin;

impl Plugin for SharedPlayerRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (animate::<Player>, update_player_animation_facing)
                    .chain()
                    .before(RenderSystems::ExtractCommands),
                /*
                (
                    add_player_directional_hints,
                    update_player_directional_hints,
                )
                    .chain(),
                 */
            )
                .run_if(in_state(InGameState::InGame)),
        );
    }
}

pub fn rendering_on_player_add(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    q_player: Query<(Entity, &Position), Added<Player>>,
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
            RenderYtoZ::default(),
        ));
    }
}

pub fn update_player_animation_facing(
    mut q_animation: Query<
        (
            &CharacterFacing,
            &mut AnimationFacing,
            &mut AnimationConfig,
            &mut Sprite,
            &LinearVelocity,
        ),
        (With<Player>, Changed<CharacterFacing>),
    >,
) {
    for (c_facing, mut facing, mut config, mut sprite, velo) in &mut q_animation {
        // TODO: Move this elsewhere
        if velo.0 == Vec2::ZERO {
            config.frame_timer.pause()
        } else if config.frame_timer.is_paused() {
            config.frame_timer.unpause()
        }
        facing.update_facing(c_facing.c_dir, &mut config, &mut sprite)
    }
}

#[derive(Component, Debug, Reflect)]
#[relationship(relationship_target = HasDirectionalHints)]
pub struct PlayerDirectionalHint {
    #[relationship]
    entity: Entity,
    targeting: Option<Entity>,
}

#[derive(Component, Debug, Clone, Reflect)]
#[relationship_target(relationship = PlayerDirectionalHint)]
pub struct HasDirectionalHints(Vec<Entity>);

pub fn add_player_directional_hints(
    mut commands: Commands,
    q_player: Query<(Entity, &Position), (With<Player>, Without<HasDirectionalHints>)>,
    q_interactables: Query<(Entity, &Position), With<Interactable>>,
) {
    for (p_ent, p_pos) in &q_player {
        let list = q_interactables.iter().collect::<Vec<(Entity, &Position)>>();
        let closest = find_closest_in_list(3, p_pos.0, &list);
        for (i_ent, _dist) in closest {
            info!("Adding Hint leading to {:?} attached to {:?}", i_ent, p_ent);
            commands.spawn((
                PlayerDirectionalHint {
                    targeting: Some(i_ent),
                    entity: p_ent,
                },
                *p_pos,
            ));
        }
    }
}

pub fn update_player_directional_hints(
    mut gizmos: Gizmos,
    q_player: Query<
        &Position,
        (
            With<Player>,
            Without<Interactable>,
            Without<PlayerDirectionalHint>,
        ),
    >,
    q_hints: Query<&PlayerDirectionalHint, (Without<Player>)>,
    q_interactables: Query<&Position, (With<Interactable>, Without<Player>)>,
) {
    for hint in &q_hints {
        let player_pos = q_player.get(hint.entity).expect("Player Not found!");
        let interactable_pos = q_interactables
            .get(hint.targeting.unwrap())
            .expect("interactable not found");

        let dir = (interactable_pos.0 - player_pos.0).normalize_or_zero();

        let draw_arrow_to = player_pos.0 + dir * 20.0;
        trace!("Drawing arrow to {:?}", draw_arrow_to);
        gizmos.arrow_2d(
            (player_pos.0 + dir * 5.0),
            draw_arrow_to,
            Color::srgb(1.0, 0.0, 0.0),
        );
    }
}
