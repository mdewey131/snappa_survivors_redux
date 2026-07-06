use bevy::prelude::*;
use lightyear::prelude::Controlled;

use crate::shared::{game_kinds::SinglePlayer, players::Player};

pub const FREE_CAM_SPEED: f32 = 10.0;

pub struct GameCameraPlugin;
impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) {
        app;
    }
}

/// Responsible for handling the gameplay, tracking the
/// player (in client) and operating more as a freecam (dedicated server)
#[derive(Component, Debug, Default)]
pub struct GameMainCamera {
    pub mode: GameCameraMode,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum GameCameraMode {
    #[default]
    FreeCam,
    Following(Entity),
}

pub fn start_camera_follow_on_controlled_player_add(
    q_player: Query<Entity, (With<Player>, Or<(Added<Controlled>, Added<SinglePlayer>)>)>,
    mut q_camera: Single<&mut GameMainCamera>,
) {
    for e in &q_player {
        q_camera.mode = GameCameraMode::Following(e);
    }
}

pub fn update_camera_pos_client(
    mut q_camera: Single<(&mut Transform, &GameMainCamera)>,
    q_following: Query<&Transform, (With<Player>, Without<GameMainCamera>)>,
) {
    if let GameCameraMode::Following(e) = q_camera.1.mode
        && let Ok(pt) = q_following.get(e)
    {
        q_camera.0.translation = (pt.translation.xy()).extend(q_camera.0.translation.z)
    }
}

pub fn update_free_cam_position(
    input: Res<ButtonInput<KeyCode>>,
    mut q_camera: Single<(&mut Transform, &GameMainCamera)>,
) {
    if let GameCameraMode::Following(_) = q_camera.1.mode {
        return;
    }
    let mut to_move = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) {
        to_move += Vec2::Y
    };
    if input.pressed(KeyCode::KeyA) {
        to_move += Vec2::NEG_X
    }
    if input.pressed(KeyCode::KeyS) {
        to_move += Vec2::NEG_Y
    }
    if input.pressed(KeyCode::KeyD) {
        to_move += Vec2::X
    }

    q_camera.0.translation += (to_move * FREE_CAM_SPEED).extend(0.0)
}
