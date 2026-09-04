use std::f32::consts::PI;

use crate::{
    render::animation::AnimationConfig,
    shared::{damage::HealthChangeResult::Invulnerable, despawn_timer::DespawnTimer},
};

use super::*;
use avian2d::{
    collision::collider::{Collider, Sensor},
    dynamics::rigid_body::RigidBody,
    parry::math::VectorExt,
};
use bevy::{
    asset::HandleTemplate, ecs::relationship::Relationship, image::TextureAtlasTemplate, prelude::*,
};

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct PaddleBack;

pub fn paddle_back() -> impl Scene {
    bsn! {
        #PaddleBack
        Ability
        PaddleBack
        Damage(5.0)
        AutoCast
        CompletesInstantly
        HoldsCharges::new(3, 0.5)
        CooldownRate(8.0)
        EffectSize(200.0)
        ProjectileCount(3.0)
        RemoveChargeOnActivation
        HasValidators [
            AbilityHolderReceivingDamage,
            //AbilityHolderNotInvulnerable,
            HasCharges
        ]
        on(paddle_back_activate)
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct PaddleBackDamageCone {
    angle: f32,
    direction: Vec2,
}

impl Default for PaddleBackDamageCone {
    fn default() -> Self {
        Self {
            angle: PI / 4.0,
            direction: Vec2::X,
        }
    }
}

/// A validator whose responsibility is to warn that the holder of this ability has incoming damage in their DamageBuffer
///
/// This is used for the PaddleBack ability
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(AbilityValidator = AbilityValidator::default())]
pub struct AbilityHolderReceivingDamage;

pub fn ability_holder_has_damage(
    mut q_validator: Query<
        (&mut AbilityValidator, &ValidatorOf),
        With<AbilityHolderReceivingDamage>,
    >,
    q_ability_step: Query<&AbilityStep>,
    q_ability: Query<&AbilityOf>,
    q_holder: Query<&HealthBuffer>,
) {
    for (mut validator, v_of) in &mut q_validator {
        let ability_to_target = if let Ok(step) = q_ability_step.get(v_of.get()) {
            step.step_of
        } else {
            v_of.get()
        };

        let holder = q_ability.get(ability_to_target).expect("this should exist");
        if let Ok(buff) = q_holder.get(holder.0) {
            validator.value = buff.get_damage().len() > 0;
            if validator.value {
                info!("Validator: {:?}", validator.value);
            }
        }
    }
}

#[derive(Component, Default, Clone, Copy, Debug)]
#[require(AbilityValidator = AbilityValidator::default())]
pub struct AbilityHolderNotInvulnerable;
fn ablity_holder_not_invuln(
    mut q_validator: Query<
        (&mut AbilityValidator, &ValidatorOf),
        With<AbilityHolderNotInvulnerable>,
    >,
    q_step: Query<&AbilityStep>,
    q_ability: Query<&AbilityOf>,
    q_holder: Query<(), With<InvulnMarker>>,
) {
    for (mut validator, v_of) in &mut q_validator {
        let entity_to_check = if let Ok(step) = q_step.get(v_of.get()) {
            let ability = q_ability.get(step.step_of).expect("Holder Not Found?");
            q_ability.get(ability.0).expect("Ability not found!").0
        } else {
            q_ability.get(v_of.get()).expect("Ability not found!").0
        };

        validator.value = q_holder.get(entity_to_check).is_ok()
    }
}

fn paddle_back_activate(
    on: On<ActivateAbility>,
    mut commands: Commands,
    q_positions: Query<(&Position, &HealthBuffer)>,
    q_ability: Query<(&Damage, &AbilityOf, &EffectSize), With<PaddleBack>>,
) {
    if let Ok((d, holder, e_size)) = q_ability.get(on.entity) {
        info!("Firing Paddle Back");
        let (Position(hold_pos), holder_buffer) = q_positions
            .get(holder.0)
            .expect("holder is missing a position or an hp buffer");
        let damaging_ent = if let Some(dam) = holder_buffer.get_damage().first() {
            dam.source
        } else {
            warn!("paddle back not firing - damage not found");
            return;
        };
        let (target_pos, _) = q_positions
            .get(damaging_ent)
            .expect("damaging ent is missing a position or an hp buffer");
        // figure out the angle and the position to spawn the thing.
        let offset = 10.0;
        let angle = hold_pos.angle(target_pos.0);
        let spawn_pos = (hold_pos + Vec2::from_angle(angle) * offset);

        commands.spawn_scene(paddle_back_cone(spawn_pos, angle, e_size.0));
        // Also, attach the invulnerability to the holder
        commands.entity(holder.0).insert(InvulnMarker::default());
    }
}
#[derive(Component, Debug, Clone, Reflect)]
pub struct InvulnMarker(pub Timer);
impl Default for InvulnMarker {
    fn default() -> Self {
        Self(Timer::from_seconds(0.5, TimerMode::Once))
    }
}
pub fn debug_tick_invuln_timer(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut q_timer: Query<(Entity, &mut InvulnMarker)>,
) {
    for (ent, mut timer) in &mut q_timer {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(ent).remove::<InvulnMarker>();
        }
    }
}
fn texture_atlas_template(
    layout: HandleTemplate<TextureAtlasLayout>,
    index: usize,
) -> TextureAtlasTemplate {
    TextureAtlasTemplate {
        layout: layout.into(),
        index,
    }
}

/// We expect size to express the radius of the circle that is being sliced out for the purposes of understanding the
/// cone
fn paddle_back_cone(pos: Vec2, angle: f32, size: f32) -> impl Scene {
    let direction = Vec2::from_angle(angle);
    // The pie slice is used to determine how big the pie should be, but under the hood this is a full circle collider.
    // We do not create a collider from the mesh, becuase that makes too complicated of a shape for testing in a performant manner
    let pie_slice = CircularSector::from_radians(size, angle);

    bsn! {
        #PaddleBackCone
        PaddleBackDamageCone {
            angle,
            direction
        }
        Sprite {
            image: "weapons/paddle_back/texture_spritesheet.png",
            texture_atlas: Option::Some(texture_atlas_template(
                 asset_value(TextureAtlasLayout::from_grid(UVec2 { x: 64, y: 64 },7, 3, None, None)),
                 0
            )),
            image_mode: SpriteImageMode::Scale(SpriteScalingMode::FillStart)
        }
        AnimationConfig::new(0,20,120)
        Mesh2d(asset_value(pie_slice))
        Position(pos)
        DespawnTimer::new(1.0)
        RigidBody
        Collider::circle(size)
        Sensor
    }
}

fn check_cone_collisions() {}

//fn spawn_paddle_back_projectile

/*
fn paddle_back_activate(
    on: Trigger<ActivateAbility>,
    q_paddle: Query<
)
*/
