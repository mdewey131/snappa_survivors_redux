use crate::{
    render::RenderYtoZ,
    shared::{
        colliders::CollisionEffect,
        combat::CombatEntityActive,
        damage::{DamageBuffer, DamageInstance, Dead},
        despawn_timer::DespawnTimer,
        game_object_spawning::SpawnGameObject,
        weapons::ActivateWeapon,
    },
};
use avian2d::prelude::*;
use bevy::{
    ecs::{entity::MapEntities, query::QueryFilter},
    prelude::*,
};
use lightyear::prelude::*;
use rand::prelude::SliceRandom;

use super::*;
const ATTACK_DISTANCE_THRESHOLD: f32 = 10.0;

pub struct ShiftyShotProtocolPlugin;

impl Plugin for ShiftyShotProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<ShiftyShotAttack>()
            .add_prediction()
            .add_map_entities();
    }
}

/// The component that describes the shifty shot weapon.
///
/// This weapon scales from:
///
/// ProjectileSpeed
/// ProjectileBounces
/// ProjectileCount,
/// Damage
/// Crit
/// Cooldown
#[derive(Component)]
pub struct WeaponShiftyShot;

pub fn shifty_shot_activate<QF: QueryFilter>(
    trigger: On<ActivateWeapon>,
    mut commands: Commands,
    q_weapon: Query<
        (
            &ChildOf,
            &ProjectileSpeed,
            &Damage,
            &ProjectileBounces,
            &AttackRange,
        ),
        (QF, With<WeaponShiftyShot>),
    >,
    q_parent: Query<&Position, Without<Enemy>>,
    q_enemies: Query<(Entity, &Position), (With<Enemy>, CombatEntityActive)>,
) {
    if let Ok((parent, speed, damage, bounces, range)) = q_weapon.get(trigger.entity) {
        let player_pos = q_parent.get(parent.0).unwrap();
        let enemy_vec = q_enemies.iter().collect::<Vec<(Entity, &Position)>>();
        let closest_enemy = find_closest_in_list(1, player_pos.0, &enemy_vec);
        if let Some(e) = closest_enemy.first() {
            if e.1 > range.0 {
            } else {
                let rem_bounces = bounces.0 as u8;
                let enemy_pos = q_enemies.get(e.0).unwrap().1;
                let init_dir = (enemy_pos.0 - player_pos.0).normalize_or_zero();
                let init_vel = speed.0 * init_dir;
                commands.queue(SpawnGameObject::new(
                    MultiPlayerComponentOptions::PREDICTED,
                    (
                        ShiftyShotAttack {
                            target: e.0,
                            remaining_bounces: rem_bounces,
                        },
                        *player_pos,
                        LinearVelocity(init_vel),
                        *damage,
                        *speed,
                        *range,
                    ),
                ));
            }
        }
    }
}

pub fn shifty_shot_deactivate<QF: QueryFilter>(
    trigger: On<DeactivateWeapon>,
    mut commands: Commands,
    mut q_weapon: Query<(&WeaponShiftyShot, &CooldownRate), QF>,
) {
    if let Ok((weapon, cdr)) = q_weapon.get_mut(trigger.entity) {
        commands.entity(trigger.entity).insert(Cooldown::new(cdr.0));
    }
}

pub fn update_shifty_shot_attack<QF: QueryFilter>(
    mut commands: Commands,
    mut q_attack: Query<
        (
            Entity,
            &mut LinearVelocity,
            &Position,
            &mut ShiftyShotAttack,
            &Damage,
            &AttackRange,
            &ProjectileSpeed,
        ),
        (QF, Without<Enemy>),
    >,
    q_enemies: Query<(Entity, &Position), (With<Enemy>, CombatEntityActive)>,
    mut q_enemy_damage: Query<&mut DamageBuffer, With<Enemy>>,
) {
    for (attack_ent, mut velo, pos, mut attack_data, dam, range, p_speed) in &mut q_attack {
        let mut should_retarget = false;
        let enemy_data = q_enemies.get(attack_data.target);
        // Enemy could die while this is in flight
        if let Ok((e_ent, enemy_pos)) = enemy_data {
            let direction_vec = (enemy_pos.0 - pos.0).normalize_or_zero();
            let new_vec = direction_vec * p_speed.0;
            velo.0 = new_vec;

            if pos.0.distance(enemy_pos.0) <= ATTACK_DISTANCE_THRESHOLD {
                let mut buffer = q_enemy_damage
                    .get_mut(e_ent)
                    .expect("Enemy without damage buffer");
                buffer.push(DamageInstance {
                    damage_source: attack_ent,
                    amount: dam.0,
                });
                if attack_data.remaining_bounces >= 1 {
                    attack_data.remaining_bounces -= 1;
                    should_retarget = true;
                } else {
                    commands.entity(attack_ent).despawn();
                }
            }
        } else {
            should_retarget = true;
        }

        if should_retarget {
            let enemy_vec = q_enemies.iter().collect::<Vec<(Entity, &Position)>>();
            // Find a few different options potentially in the area for variety
            let closest_2 = find_closest_in_list(5, pos.0, &enemy_vec);
            let mut filtered_list = closest_2
                .into_iter()
                .filter(|record| (record.0 != attack_data.target) && (record.1 <= range.0))
                .map(|record| record.0)
                .collect::<Vec<Entity>>();

            filtered_list.shuffle(&mut rand::rng());
            // There could be no one!
            if let Some(new_enemy_ent) = filtered_list.first() {
                attack_data.target = *new_enemy_ent;
                // It could be the case that we previously gave this a despawn timer, but
                // "now" there's an enemy and we just found them, so we always do this, but it
                // will sometimes do nothing
                commands.entity(attack_ent).remove::<DespawnTimer>();
            } else {
                velo.0 = Vec2::ZERO;
                commands.entity(attack_ent).insert(DespawnTimer::new(0.5));
            }
        }
    }
}

/// Describes the shot fired by a shifty shot weapon,
/// this stores reference to the entity its supposed to be targeting in order to know when to bounce
/// We don't handle this with collisions because it gets real hard to manage real fast, and it's likely going to be chaos to
/// orchestrate without causing many bugs
#[derive(Component, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Reflect)]
#[require(RigidBody::Dynamic)]
pub struct ShiftyShotAttack {
    pub target: Entity,
    pub remaining_bounces: u8,
}

impl MapEntities for ShiftyShotAttack {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        self.target = entity_mapper.get_mapped(self.target);
    }
}

pub fn add_shifty_shot_attack_sprite<QF: QueryFilter>(
    trigger: On<Add, ShiftyShotAttack>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    q_attack: Query<(&Position), (With<ShiftyShotAttack>, QF)>,
) {
    if let Ok(pos) = q_attack.get(trigger.entity) {
        let image: Handle<Image> = assets.load("weapons/shifty_shot/projectile.png");
        commands.entity(trigger.entity).insert((
            Sprite::from(image),
            Transform::from_translation(pos.0.extend(pos.0.y)),
            RenderYtoZ::default(),
        ));
    }
}
