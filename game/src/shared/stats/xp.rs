use bevy::{ecs::query::QueryFilter, prelude::*};
use serde::{Deserialize, Serialize};

use crate::shared::{
    game_kinds::{CurrentGameKind, MultiPlayerComponentOptions},
    game_object_spawning::spawn_game_object,
    players::Player,
    stats::components::XPGain,
};

#[derive(Component, Debug, Clone, Serialize, Deserialize, PartialEq, Reflect)]
pub struct LevelManager {
    pub c_level: u8,
    pub c_xp: f32,
    /// For display purposes
    pub prev_max: f32,
    pub next_max: f32,
}

impl Default for LevelManager {
    fn default() -> Self {
        Self {
            c_level: 1,
            c_xp: 0.0,
            prev_max: 0.0,
            next_max: 5.0,
        }
    }
}

#[derive(Message)]
pub struct ApplyXPMessage {
    pub amount: f32,
}

pub fn add_xp<QF: QueryFilter>(
    mut mess: MessageReader<ApplyXPMessage>,
    mut q_level: Single<&mut LevelManager>,
    _q_stats: Query<&XPGain, (With<Player>, QF)>,
) {
    for xp in mess.read() {
        q_level.c_xp += xp.amount;
        if q_level.c_xp >= q_level.next_max {
            q_level.c_level += 1;
            q_level.prev_max = q_level.next_max;
            q_level.next_max = (q_level.c_level as f32 * 10.0).powf(1.5);
        }
    }
}

pub fn add_level_manager(mut commands: Commands, gk: Res<CurrentGameKind>) {
    let _lm = spawn_game_object(
        &mut commands,
        gk.0.unwrap(),
        MultiPlayerComponentOptions {
            pred: true,
            interp: false,
        },
        (LevelManager::default(), Name::from("Level Manager")),
    );
}
