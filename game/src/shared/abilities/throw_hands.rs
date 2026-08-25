use crate::{render::RenderYtoZ, shared::combat::CombatEntityActive};
use lightyear::prelude::PredictionBuilderExt;

use super::*;
use bevy::{
    ecs::{entity::MapEntities, template::EntityTemplate},
    prelude::*,
};

use lightyear::prelude::AppComponentExt;
use serde::Serialize;

const BASE_WINDUP_TIME: f32 = 0.5;
const BASE_WINDDOWN_TIME: f32 = 0.5;
const TIME_BETWEEN_THROW_HANDS_ATTACKS: f32 = 0.25;

pub struct ThrowHandsPlugin;
impl Plugin for ThrowHandsPlugin {
    fn build(&self, app: &mut App) {
        app;
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect, Default)]
pub struct ThrowHands {
    pub targets: Option<Vec<Entity>>,
    pub current: u8,
}

pub fn throw_hands<C: Component>() -> impl Scene {
    bsn! {
        #ThrowHands
        ThrowHands
        AutoCast
        Ability
        PulseActivation {timer: Timer::from_seconds(TIME_BETWEEN_THROW_HANDS_ATTACKS, TimerMode::Repeating)}
        Activations {
            max: Option::Some(1)
        }
        AddCooldownOnCompletion
        Cooldown::new(0.25)
        ProjectileCount(1.0)
        AttackRange(500.0)
        Damage(4.0)
        CooldownRate(5.0)
        CritChance(0.15)
        CritDamage(1.5)
        HasValidators [
            AbilityOffCooldown,
            EnemyInAttackRange
        ]
        on(throw_hands_activate::<C>)
        on(throw_hands_deactivate)
    }
}

fn throw_hands_activate<T: Component>(
    on: On<ActivateAbility>,
    mut commands: Commands,
    mut q_hands: Query<
        (
            &AbilityOf,
            &mut ThrowHands,
            &Damage,
            &AttackRange,
            &ProjectileCount,
            &CritChance,
            &CritDamage,
        ),
        With<ThrowHands>,
    >,
    q_holder: Query<&Position, Without<T>>,
    q_targets: Query<(Entity, &Position), (With<T>, CombatEntityActive)>,
) {
    if let Ok((holder, mut throw, damage, range, p_count, cc, cd)) = q_hands.get_mut(on.entity) {
        let holder = q_holder.get(holder.0).expect("This player should exist");
        let target_vec = q_targets.iter().collect::<Vec<(Entity, &Position)>>();
        if throw.targets.is_none() {
            let targets = find_closest_in_list(p_count.0 as u8, holder.0, &target_vec);
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
        let t_pos = q_targets.get(target).unwrap().1;
        commands.spawn_scene(throw_hands_attack(target, t_pos.0, damage.0, cc.0, cd.0));
    }
}

fn throw_hands_deactivate(on: On<DeactivateAbility>, mut q_throw_hands: Query<&mut ThrowHands>) {
    if let Ok(mut hands) = q_throw_hands.get_mut(on.entity) {
        hands.targets = Option::None;
        hands.current = 0;
    }
}

pub fn throw_hands_attack(target: Entity, pos: Vec2, damage: f32, cc: f32, cd: f32) -> impl Scene {
    let attack_pos = pos + (Vec2::Y * 10.0);
    bsn! {
        #ThrowHandsAttack
        ThrowHandsAttack {target}
        Damage(damage)
        CritChance(cc)
        CritDamage(cd)
        Damage(5.0)
        Sprite {image: "weapons/throw_hands/attack.png"}
        Ability
        Position(attack_pos)
        RenderYtoZ::new(10.0)
        HasAbilitySteps [
            (
                #ThrowHandsWindupStep
                Ability
                AutoCast
                ActiveForTime(Timer::from_seconds(BASE_WINDUP_TIME, TimerMode::Once))
            ),
            (
                #ThrowHandsAttackStep
                Ability
                AutoCast
                DamageTargetsOnCompletion(vec![target])
                CompletesInstantly
                HasValidators [
                    StepCompleted(#ThrowHandsWindupStep)
                ]
            ),
            (
                #ThrowHandsWindDownStep
                Ability
                AutoCast
                ActiveForTime(Timer::from_seconds(BASE_WINDDOWN_TIME , TimerMode::Once))
                HasValidators [
                    StepCompleted(#ThrowHandsAttackStep)
                ]
                DespawnOnCompletion
            ),

        ]
    }
}

pub struct ThrowHandsProtocolPlugin;

impl Plugin for ThrowHandsProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.component::<ThrowHandsAttack>().predict();
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect, PartialEq, FromTemplate)]
pub struct ThrowHandsAttack {
    pub target: Entity,
}

impl ThrowHandsAttack {
    fn new(target: Entity) -> Self {
        Self { target }
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

/*
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
    mut q_attack: Query<
        (
            Entity,
            &mut ThrowHandsAttack,
            &Damage,
            &CritChance,
            &CritDamage,
        ),
        QF,
    >,
    mut q_target: Query<&mut HealthBuffer>,
) {
    for (attack_ent, mut throw, damage, cc, cd) in &mut q_attack {
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
                    t_buffer.push_damage(attack_ent, damage.0, Some((cc.0, cd.0)));
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
 */
