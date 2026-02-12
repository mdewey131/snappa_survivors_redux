use crate::shared::{
    combat::Cooldown, enemies::Enemy, players::Player, stats::components::*,
    weapons::DeactivateWeapon,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::ActivateWeapon;

pub struct SharedThrowHandsPlugin;
impl Plugin for SharedThrowHandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_activate).add_observer(on_deactivate);
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ThrowHands {
    pub targets: Option<Vec<Entity>>,
    pub current: u8,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ThrowHandsAttack {
    pub targeting: Entity,
}

pub fn on_activate(
    trigger: On<ActivateWeapon>,
    mut commands: Commands,
    mut q_weapon: Query<(&ChildOf, &mut ThrowHands, &ProjectileCount)>,
    q_player: Query<&Position, With<Player>>,
    q_enemy: Query<(Entity, &Position), (With<Enemy>, Without<Player>)>,
) {
    if let Ok((child, mut throw, p_count)) = q_weapon.get_mut(trigger.entity) {
        let p_pos = q_player.get(child.0).expect("This player should exist");
        if throw.targets.is_none() {
            let targets = find_targets(p_count.0 as u8, p_pos.0, &q_enemy);
            throw.targets = Some(targets);
        }
        let m_target = throw.targets.as_ref().unwrap().get(throw.current as usize);

        // CHECK THIS LINE FOR WEIRD BEHAVIOR IN THIS WEAPON
        let target = if let Some(t) = m_target {
            t.clone()
        } else {
            return;
        };

        throw.current += 1;
        commands.spawn(ThrowHandsAttack { targeting: target });
    }
}

fn find_targets(
    num_to_find: u8,
    player_pos: Vec2,
    q_enemy_pos: &Query<(Entity, &Position), (With<Enemy>, Without<Player>)>,
) -> Vec<Entity> {
    let mut sorted = q_enemy_pos
        .iter()
        .map(|(ent, pos)| {
            let dist = pos.0.distance(player_pos);
            (ent, pos, dist)
        })
        .collect::<Vec<(Entity, &Position, f32)>>();

    sorted.sort_by(|q1, q2| q1.2.partial_cmp(&q2.2).unwrap());

    let mut targets = Vec::new();
    for _i in (0..num_to_find) {
        let m_enemy = sorted.pop();
        if let Some(record) = m_enemy {
            targets.push(record.0)
        }
    }
    targets
}

pub fn on_deactivate(
    trigger: On<DeactivateWeapon>,
    mut commands: Commands,
    mut q_weapon: Query<(&mut ThrowHands, &CooldownRate)>,
) {
    if let Ok((mut weapon, cdr)) = q_weapon.get_mut(trigger.entity) {
        weapon.targets = None;
        weapon.current = 0;
        commands.entity(trigger.entity).insert(Cooldown::new(cdr.0));
    }
}
