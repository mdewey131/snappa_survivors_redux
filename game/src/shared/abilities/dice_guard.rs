use crate::{shared::game_kinds::SinglePlayer, utils::SpawnedBy};

use super::*;
use bevy::prelude::*;

#[derive(Component, Default, Clone, Reflect)]
pub struct DiceGuard {
    pub dice: Option<Vec<Entity>>,
}

pub fn dice_guard() -> impl Scene {
    bsn! {
        #DiceGuard
        Ability
        AutoCast
        AttackRange(50.0)
        DiceGuard
        AddCooldownOnCompletion
        CooldownRate(9.0)
        Cooldown::new(0.25)
        Damage(9.0)
        EffectDuration(6.0)
        EffectSize(20.0)
        ProjectileCount(2.0)
        ProjectileSpeed(30.0)
        SinglePlayer
        HasValidators [
            #OffCD
            AbilityOffCooldown
        ]
        on(dice_guard_activate)
        on(dice_guard_deactivate)

    }
}

#[derive(Component, Debug, Serialize, Deserialize, PartialEq, Clone, Copy, Default)]
pub struct DiceGuardProjectile;

pub fn dice_guard_projectile(
    ability_holder: Entity,
    holder_pos: Vec2,
    ability_ent: Entity,
    radius: f32,
    angle: f32,
    speed: f32,
    damage: f32,
    effect_size: f32,
) -> impl Scene {
    let pos = holder_pos + Vec2::from_angle(angle) * radius;
    bsn! {
        #DGProjectile
        Sprite {
            image: "weapons/dice_guard/projectile.png"
        }
        DiceGuardProjectile
        Damage(damage)
        EffectSize(effect_size)
        SinglePlayer
        Projectile {
            movement: ProjectileMovement::Orbital {
                around: ability_holder,
                speed: speed,
                c_angle: angle,
                radius: radius,
            }
        }
        Position(pos)
        AppliesCollisionEffect::<ApplyDamage>::new(
            [ColliderTypes::Enemy].into(),
            ApplyDamage::default(),
        )
    }
}

pub fn dice_guard_activate(
    on: On<ActivateAbility>,
    mut commands: Commands,
    game_kind: Res<CurrentGameKind>,
    mut q_dice_guard: Query<(
        Entity,
        &mut DiceGuard,
        &AttackRange,
        &AbilityOf,
        &ProjectileCount,
        &EffectSize,
        &ProjectileSpeed,
        &Damage,
        &EffectDuration,
    )>,
    q_holder: Query<&Position>,
) {
    if let Ok((dg_ent, mut dg, range, holder, p_count, eff_size, proj_speed, dam, dur)) =
        q_dice_guard.get_mut(on.entity)
    {
        commands
            .entity(dg_ent)
            .insert(ActiveForTime(Timer::from_seconds(dur.0, TimerMode::Once)));
        let mut projectiles = vec![];
        info!("Dice guard activated!");
        let holder_pos = q_holder.get(holder.0).unwrap();

        let iters = p_count.0.floor() as usize;
        let r = range.0 * iters as f32;

        for i in 0..iters {
            let angle = std::f32::consts::TAU * (i as f32 / p_count.0);
            let projectile = commands
                .spawn_scene(dice_guard_projectile(
                    holder.0,
                    holder_pos.0,
                    dg_ent,
                    r,
                    angle,
                    proj_speed.0,
                    dam.0,
                    eff_size.0,
                ))
                .id();

            projectiles.push(projectile);
        }
        dg.dice = Some(projectiles);
        /*

            // Shorhand for now
            //spawn_positions.positions_2d().into_iter().enumerate() {
            let proj = Projectile {
                movement: ProjectileMovement::Orbital {
                    around: holder.0,
                    speed: proj_speed.0,
                    c_angle: angle,
                    radius: r,
                },
            };
            let pos = holder_pos.0 + Vec2::from_angle(angle) * r;

            trace!("Found angle to be {angle}, position is {:?}", pos);
            let ent = spawn_game_object(
                &mut commands,
                game_kinds::GameKinds::SinglePlayer,
                //game_kind.0.unwrap(),
                None::<()>,
                MultiPlayerComponentOptions::from(proj),
                (
                    proj,
                    DiceGuardProjectile,
                    Position(pos),
                    CreatedBy(holder.0),
                    *dam,
                    *eff_size,
                    AppliesCollisionEffect::new(
                        [ColliderTypes::Enemy].into(),
                        ApplyDamage::default(),
                    ),
                ),
            );
        }
        */
    }
}

pub fn dice_guard_deactivate(
    on: On<DeactivateAbility>,
    mut commands: Commands,
    mut q_ability: Query<&mut DiceGuard>,
) {
    if let Ok(mut dg) = q_ability.get_mut(on.entity) {
        if let Some(ref list) = dg.dice {
            for projectile in list.iter() {
                commands.entity(*projectile).despawn();
            }
        }
        dg.dice = None;
    }
}
