use crate::shared::{
    combat::{CombatSystemSet, Cooldown},
    damage::{DamageBuffer, DamageInstance},
    enemies::Enemy,
    game_kinds::{CurrentGameKind, MultiPlayerComponentOptions},
    game_object_spawning::spawn_game_object,
    players::Player,
    states::InGameState,
    stats::components::*,
    weapons::DeactivateWeapon,
};
use avian2d::prelude::*;
use bevy::{ecs::query::QueryFilter, prelude::*};
use serde::{Deserialize, Serialize};
const BASE_WINDUP_TIME: f32 = 0.2;
const BASE_WINDDOWN_TIME: f32 = 0.2;

use super::ActivateWeapon;

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ThrowHands {
    pub targets: Option<Vec<Entity>>,
    pub current: u8,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ThrowHandsAttack {
    pub target: Entity,
    pub state: ThrowHandsAttackState,
    pub timer: Timer,
}
impl ThrowHandsAttack {
    fn new(target: Entity) -> Self {
        let state = ThrowHandsAttackState::Windup;
        let timer = Self::timer_from_state(state);
        Self {
            target,
            state,
            timer,
        }
    }
    fn timer_from_state(state: ThrowHandsAttackState) -> Timer {
        let time = match state {
            ThrowHandsAttackState::Attack => unimplemented!(),
            ThrowHandsAttackState::Windup => BASE_WINDUP_TIME,
            ThrowHandsAttackState::Winddown => BASE_WINDDOWN_TIME,
        };
        Timer::from_seconds(time, TimerMode::Once)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
pub enum ThrowHandsAttackState {
    Windup,
    Attack,
    Winddown,
}

pub fn on_activate<QF: QueryFilter>(
    trigger: On<ActivateWeapon>,
    game_kind: Res<CurrentGameKind>,
    mut commands: Commands,
    mut q_weapon: Query<(&ChildOf, &mut ThrowHands, &ProjectileCount, &Damage), QF>,
    q_player: Query<&Position, With<Player>>,
    q_enemy: Query<(Entity, &Position), (With<Enemy>, Without<Player>)>,
) {
    if let Ok((child, mut throw, p_count, damage)) = q_weapon.get_mut(trigger.entity) {
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
        spawn_game_object(
            &mut commands,
            game_kind.0.unwrap(),
            None::<()>,
            MultiPlayerComponentOptions::PREDICTED,
            (ThrowHandsAttack::new(target), *damage),
        );
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
    sorted.reverse();
    let mut targets = Vec::new();
    for _i in (0..num_to_find) {
        let m_enemy = sorted.pop();
        if let Some(record) = m_enemy {
            targets.push(record.0)
        }
    }
    targets
}

pub fn on_deactivate<QF: QueryFilter>(
    trigger: On<DeactivateWeapon>,
    mut commands: Commands,
    mut q_weapon: Query<(&mut ThrowHands, &CooldownRate), QF>,
) {
    if let Ok((mut weapon, cdr)) = q_weapon.get_mut(trigger.entity) {
        weapon.targets = None;
        weapon.current = 0;
        commands.entity(trigger.entity).insert(Cooldown::new(cdr.0));
    }
}

pub fn update_attack<QF: QueryFilter>(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut q_attack: Query<(Entity, &mut ThrowHandsAttack, &Damage), QF>,
    mut q_target: Query<(&mut DamageBuffer)>,
) {
    for (attack_ent, mut throw, damage) in &mut q_attack {
        match throw.state {
            ThrowHandsAttackState::Windup => {
                throw.timer.tick(time.delta());
                if throw.timer.just_finished() {
                    throw.state = ThrowHandsAttackState::Attack
                }
            }
            ThrowHandsAttackState::Attack => {
                throw.state = ThrowHandsAttackState::Winddown;
                throw.timer = ThrowHandsAttack::timer_from_state(throw.state);
                if let Ok(mut t_buffer) = q_target.get_mut(throw.target) {
                    t_buffer.push(DamageInstance {
                        damage_source: attack_ent,
                        amount: damage.0,
                    });
                };
            }
            ThrowHandsAttackState::Winddown => {
                throw.timer.tick(time.delta());
                if throw.timer.just_finished() {
                    commands.entity(attack_ent).despawn();
                }
            }
        }
    }
}
