use bevy::{prelude::*, render::RenderSystems};

use crate::render::camera::*;

pub struct GameCameraClientPlugin;

impl Plugin for GameCameraClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                start_camera_follow_on_controlled_player_add,
                update_camera_pos_client,
            )
                .chain()
                .before(RenderSystems::ExtractCommands),
        );
    }
}
