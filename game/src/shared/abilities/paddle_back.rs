use super::*;
use bevy::{ecs::relationship::Relationship, prelude::*};

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct PaddleBack;

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct PaddleBackDamageCone;

/// A validator whose responsibility is to warn that the holder of this ability has incoming damage in their DamageBuffer
///
/// This is used for the PaddleBack ability
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(AbilityValidator = AbilityValidator::default())]
pub struct AbilityHolderHasDamage;

pub fn ability_holder_has_damage(
    mut q_validator: Query<(&mut AbilityValidator, &ValidatorOf), With<AbilityHolderHasDamage>>,
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
            info!("Validator: {:?}", validator.value);
        }
    }
}

/*
fn paddle_back_activate(
    on: Trigger<ActivateAbility>,
    q_paddle: Query<
)
*/
