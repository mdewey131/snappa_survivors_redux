use crate::{
    render::RenderYtoZ,
    shared::{
        combat::CombatEntityActive,
        damage::{DeathState, HealthBuffer, HealthChangeInstance},
        despawn_timer::DespawnTimer,
        game_object_spawning::SpawnGameObject,
        weapons::ActivateWeapon,
    },
    utils::CreatedBy,
};
use avian2d::prelude::*;
use bevy::{
    ecs::{
        entity::MapEntities,
        query::{QueryEntityError, QueryFilter},
    },
    prelude::*,
};
use lightyear::prelude::*;
use rand::prelude::SliceRandom;

use super::*;
const ATTACK_DISTANCE_THRESHOLD: f32 = 10.0;

pub struct ShiftyShotProtocolPlugin;

impl Plugin for ShiftyShotProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.component::<ShiftyShotAttack>().predict();
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
            &CritChance,
            &CritDamage,
        ),
        (QF, With<WeaponShiftyShot>),
    >,
    q_parent: Query<&Position, Without<Enemy>>,
    q_enemies: Query<(Entity, &Position), (With<Enemy>, CombatEntityActive)>,
) {
    if let Ok((parent, speed, damage, bounces, range, cc, cd)) = q_weapon.get(trigger.entity) {
        let player_pos = q_parent.get(parent.0).unwrap();
        let enemy_vec = q_enemies.iter().collect::<Vec<(Entity, &Position)>>();
        let closest_enemy = find_closest_in_list(1, player_pos.0, &enemy_vec);
        if let Some(e) = closest_enemy.first() {
            if e.1 > range.0 {
            } else {
                let rem_bounces = bounces.0 as u8;
                let enemy_pos = q_enemies.get(e.0).unwrap().1;
                let init_dir = (enemy_pos.0 - player_pos.0).normalize_or_zero();
                let init_vel = init_dir * speed.0;
                commands.queue(SpawnGameObject::new(
                    MultiPlayerComponentOptions::PREDICTED,
                    (
                        ShiftyShotAttack {
                            target: Some(e.0),
                            remaining_bounces: rem_bounces,
                        },
                        *player_pos,
                        LinearVelocity(init_vel),
                        CreatedBy(parent.0),
                        *damage,
                        *speed,
                        *range,
                        *cd,
                        *cc,
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
    if let Ok((_weapon, cdr)) = q_weapon.get_mut(trigger.entity) {
        commands.entity(trigger.entity).insert(Cooldown::new(cdr.0));
    }
}

pub fn update_shifty_shot_attack(
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
            &CritChance,
            &CritDamage,
            Has<DespawnTimer>,
        ),
        (Without<Enemy>),
    >,
    q_enemies: Query<(Entity, &Position), (With<Enemy>, CombatEntityActive)>,
    mut q_enemy_damage: Query<&mut HealthBuffer, With<Enemy>>,
) {
    for (attack_ent, mut velo, pos, mut attack_data, dam, range, p_speed, cc, cd, has_timer) in
        &mut q_attack
    {
        let mut should_retarget = false;
        let mut despawn = false;
        let enemy_data = if let Some(t) = attack_data.target {
            q_enemies.get(t).map_err(|_e| String::from("not found"))
        } else {
            Err(String::new())
        };

        // Enemy could die while this is in flight
        if let Ok((e_ent, enemy_pos)) = enemy_data {
            let direction_vec = (enemy_pos.0 - pos.0).normalize_or_zero();
            let new_vec = direction_vec * p_speed.0;
            velo.0 = new_vec;

            if pos.0.distance(enemy_pos.0) <= ATTACK_DISTANCE_THRESHOLD {
                let mut buffer = q_enemy_damage
                    .get_mut(e_ent)
                    .expect("Enemy without damage buffer");
                buffer.push_damage(attack_ent, dam.0, Some((cc.0, cd.0)));
                if attack_data.remaining_bounces >= 1 {
                    attack_data.remaining_bounces -= 1;
                    should_retarget = true;
                } else {
                    attack_data.target = None;
                    despawn = true;
                }
            }
        } else {
            should_retarget = true;
        }
        if should_retarget {
            let enemy_vec = q_enemies.iter().collect::<Vec<(Entity, &Position)>>();
            // Find a few different options potentially in the area for variety
            let closest = find_closest_in_list(5, pos.0, &enemy_vec);
            let mut filtered_list = closest
                .into_iter()
                .filter(|record| {
                    if let Some(t) = attack_data.target {
                        (record.0 != t) && (record.1 <= range.0)
                    } else {
                        false
                    }
                })
                .map(|record| record.0)
                .collect::<Vec<Entity>>();

            filtered_list.shuffle(&mut rand::rng());
            // There could be no one!
            attack_data.target = filtered_list.first().copied();
            if attack_data.target.is_none() {
                velo.0 = Vec2::ZERO;
                despawn = true;
            } else if has_timer {
                commands.entity(attack_ent).remove::<DespawnTimer>();
            }
        }
        if despawn && !has_timer {
            commands.entity(attack_ent).insert(DespawnTimer::new(0.5));
        }
    }
}

/// Describes the shot fired by a shifty shot weapon,
/// this stores reference to the entity its supposed to be targeting in order to know when to bounce
/// We don't handle this with collisions because it gets real hard to manage real fast, and it's likely going to be chaos to
/// orchestrate without causing many bugs
#[derive(Component, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Reflect)]
#[require(RigidBody::Dynamic, Name = Name::from("Shifty Shot Attack"), CombatEntity = CombatEntity)]
pub struct ShiftyShotAttack {
    pub target: Option<Entity>,
    pub remaining_bounces: u8,
}

impl MapEntities for ShiftyShotAttack {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        if let Some(ref mut t) = self.target {
            *t = entity_mapper.get_mapped(*t);
        } else {
            return;
        }
    }
}

pub fn add_shifty_shot_attack_sprite<QF: QueryFilter>(
    trigger: On<Add, ShiftyShotAttack>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    q_attack: Query<&Position, (With<ShiftyShotAttack>, QF)>,
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

/// In the event that this shot doesn't have a target (synonym: has a DespawnTimer)
/// We slowly shrink it down while its dying
///
/// If it gets a target and this component gets removed, we'll restore that size
pub fn reduce_orphaned_attack(
    mut q_sprite: Query<(&mut Transform, &DespawnTimer), With<ShiftyShotAttack>>,
) {
    for (mut pos, timer) in &mut q_sprite {
        pos.scale = Vec3::splat(timer.fraction_remaining());
    }
}

pub fn restore_attack_size_on_target_found(
    mut removed: RemovedComponents<DespawnTimer>,
    mut q_attack: Query<(&mut Transform), With<ShiftyShotAttack>>,
) {
    for rm in removed.read() {
        if let Ok(mut t) = q_attack.get_mut(rm) {
            t.scale = Vec3::splat(1.0)
        }
    }
}
