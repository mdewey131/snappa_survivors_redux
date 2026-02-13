use crate::shared::{
    combat::{CombatSystemSet, Cooldown},
    damage::{DamageBuffer, DamageInstance},
    enemies::Enemy,
    players::Player,
    states::InGameState,
    stats::components::*,
    weapons::DeactivateWeapon,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::ActivateWeapon;

pub struct SharedThrowHandsPlugin;
impl Plugin for SharedThrowHandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            update_attack
                .run_if(in_state(InGameState::InGame))
                .in_set(CombatSystemSet::Combat),
        )
        .add_observer(on_activate)
        .add_observer(on_deactivate);
    }
}

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
        Self {
            target,
            state: ThrowHandsAttackState::Windup,
            timer: Timer::from_seconds(0.5, TimerMode::Once),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum ThrowHandsAttackState {
    Windup,
    Attack,
    Winddown,
}

pub fn on_activate(
    trigger: On<ActivateWeapon>,
    mut commands: Commands,
    mut q_weapon: Query<(&ChildOf, &mut ThrowHands, &ProjectileCount, &Damage)>,
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
        commands.spawn((ThrowHandsAttack::new(target), *damage));
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

pub fn update_attack(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut q_attack: Query<(Entity, &mut ThrowHandsAttack, &Damage)>,
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
                throw.timer = Timer::from_seconds(0.2, TimerMode::Once);
                let mut t_buffer = q_target.get_mut(throw.target).expect("Not Found!");
                t_buffer.push(DamageInstance {
                    damage_source: attack_ent,
                    amount: damage.0,
                });
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
