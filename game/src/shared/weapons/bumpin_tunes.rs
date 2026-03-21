use avian2d::prelude::*;
use bevy::{ecs::query::QueryFilter, prelude::*};
use serde::{Deserialize, Serialize};

use super::ActivateWeapon;
use crate::shared::{damage::*, enemies::Enemy, stats::components::*};

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BumpinTunes;

pub fn bumpin_tunes_activate<QF: QueryFilter>(
    trigger: On<ActivateWeapon>,
    q_player: Query<&Position, Without<Enemy>>,
    q_weapon: Query<(&ChildOf, &Damage, &EffectSize), (With<BumpinTunes>, QF)>,
    mut q_enemies: Query<(&Position, &mut DamageBuffer), With<Enemy>>,
) {
    if let Ok((parent, dam, size)) = q_weapon.get(trigger.entity) {
        let player_loc = q_player.get(parent.0).expect("Player position not found!");
        for (e_pos, mut buff) in &mut q_enemies {
            if player_loc.0.distance(e_pos.0) <= size.0 {
                buff.push(DamageInstance {
                    damage_source: trigger.entity,
                    amount: dam.0,
                })
            }
        }
    }
}
