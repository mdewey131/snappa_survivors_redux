use bevy::prelude::*;
use lightyear::prelude::Controlled;

use crate::shared::players::Player;

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
    t: On<Add, Controlled>,
    q_player: Query<Has<Controlled>, With<Player>>,
    mut q_camera: Single<&mut GameMainCamera>,
) {
    if let Ok(cont) = q_player.get(t.entity) {
        if cont {
            (*q_camera).mode = GameCameraMode::Following(t.entity);
        }
    }
}

pub fn update_camera_pos_client(
    mut q_camera: Single<(&mut Transform, &GameMainCamera)>,
    q_following: Query<&Transform, (With<Player>, Without<GameMainCamera>)>,
) {
    match q_camera.1.mode {
        GameCameraMode::Following(e) => {
            if let Ok(pt) = q_following.get(e) {
                q_camera.0.translation = (pt.translation.xy()).extend(q_camera.0.translation.z)
            }
        }
        _ => {}
    }
}
