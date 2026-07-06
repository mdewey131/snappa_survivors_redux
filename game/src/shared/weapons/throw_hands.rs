use crate::shared::{
    combat::{CombatEntityActive, Cooldown},
    damage::{DamageBuffer, DamageInstance},
    enemies::Enemy,
    game_kinds::MultiPlayerComponentOptions,
    game_object_spawning::SpawnGameObject,
    stats::components::*,
    weapons::{DeactivateWeapon, find_closest_in_list},
};
use avian2d::prelude::*;
use bevy::{
    ecs::{entity::MapEntities, query::QueryFilter},
    prelude::*,
};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
const BASE_WINDUP_TIME: f32 = 0.2;
const BASE_WINDDOWN_TIME: f32 = 0.2;

use super::ActivateWeapon;

pub struct ThrowHandsProtocolPlugin;

impl Plugin for ThrowHandsProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<ThrowHandsAttack>()
            .add_prediction()
            .add_map_entities();
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ThrowHands {
    pub targets: Option<Vec<Entity>>,
    pub current: u8,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect, PartialEq)]
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
impl MapEntities for ThrowHandsAttack {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        self.target = entity_mapper.get_mapped(self.target);
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Reflect, PartialEq)]
pub enum ThrowHandsAttackState {
    Windup,
    Attack,
    Winddown,
}

pub fn on_activate<QF: QueryFilter>(
    trigger: On<ActivateWeapon>,
    mut commands: Commands,
    mut q_weapon: Query<
        (
            &ChildOf,
            &mut ThrowHands,
            &ProjectileCount,
            &Damage,
            &AttackRange,
        ),
        QF,
    >,
    q_player: Query<&Position, Without<Enemy>>,
    q_enemy: Query<(Entity, &Position), (With<Enemy>, CombatEntityActive)>,
) {
    if let Ok((child, mut throw, p_count, damage, range)) = q_weapon.get_mut(trigger.entity) {
        let p_pos = q_player.get(child.0).expect("This player should exist");
        let enemy_vec = q_enemy.iter().collect::<Vec<(Entity, &Position)>>();
        if throw.targets.is_none() {
            let targets = find_closest_in_list(p_count.0 as u8, p_pos.0, &enemy_vec);
            let ts = targets
                .iter()
                .filter_map(|record| {
                    if record.1 <= range.0 {
                        Some(record.0)
                    } else {
                        None
                    }
                })
                .collect::<Vec<Entity>>();
            throw.targets = Some(ts);
        }
        let m_target = throw.targets.as_ref().unwrap().get(throw.current as usize);

        // CHECK THIS LINE FOR WEIRD BEHAVIOR IN THIS WEAPON
        let target = if let Some(t) = m_target {
            *t
        } else {
            return;
        };

        throw.current += 1;
        commands.queue(SpawnGameObject::new(
            MultiPlayerComponentOptions::PREDICTED,
            (ThrowHandsAttack::new(target), *damage),
        ));
    }
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
    mut q_target: Query<&mut DamageBuffer >,
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
