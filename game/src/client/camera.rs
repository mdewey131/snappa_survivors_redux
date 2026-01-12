use bevy::{prelude::*, render::RenderSystems};

use crate::render::camera::*;

pub struct GameCameraClientPlugin;

impl Plugin for GameCameraClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            update_camera_pos_client.before(RenderSystems::ExtractCommands),
        )
        .add_observer(start_camera_follow_on_controlled_player_add);
    }
}
