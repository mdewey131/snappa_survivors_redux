use bevy::{image::TextureError::InvalidImageExtension, prelude::*};

use super::*;
/// An individual entity that is responsible for saying whether or not this ability can proceed.
/// Validators run before systems that check whether or not something should be allowed to move from
/// Requested to Executing
#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
pub struct AbilityValidator {
    pub value: bool,
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[relationship_target(relationship = ValidatorOf, linked_spawn)]
pub struct HasValidators(Vec<Entity>);

#[derive(Component, Reflect, Debug, Clone)]
#[relationship(relationship_target = HasValidators)]
pub struct ValidatorOf {
    #[relationship]
    entity: Entity,
}

#[derive(Component, Debug, Clone, Copy, Default)]
#[require(AbilityValidator = AbilityValidator::default())]
pub struct AbilityOffCooldown;

pub fn check_cooldown_validator(
    mut q_validator: Query<(&mut AbilityValidator, &ValidatorOf), With<AbilityOffCooldown>>,
    q_step: Query<&AbilityStep>,
    q_stat_holder_abilities: Query<(), (With<Ability>, Without<Cooldown>)>,
) {
    for (mut validator, holder) in &mut q_validator {
        let entity_to_check = if let Ok(step) = q_step.get(holder.entity) {
            step.step_of
        } else {
            holder.entity
        };
        let off_cooldown = if q_stat_holder_abilities.get(entity_to_check).is_ok() {
            true
        } else {
            false
        };
        validator.value = off_cooldown
    }
}

/// A validator returning true if at least one enemy is in range
#[derive(Component, Debug, Clone, Default)]
#[require(AbilityValidator = AbilityValidator::default())]
pub struct EnemyInAttackRange;
pub fn enemy_in_attack_range(
    mut q_validator: Query<(&mut AbilityValidator, &ValidatorOf), With<EnemyInAttackRange>>,
    q_ability: Query<(
        &Ability,
        Option<&AbilityOf>,
        Option<&AttackRange>,
        Option<&AbilityStep>,
    )>,
    q_transforms: Query<(Entity, &Position, Option<&Enemy>)>,
) {
    // first, find the holder of this ability
    for (mut validator, v_of) in &mut q_validator {
        let validator_ability = q_ability.get(v_of.entity).expect("This should exist");
        let (holder_ent, range) = if validator_ability.3.is_none() {
            (validator_ability.1.unwrap(), validator_ability.2.unwrap())
        } else {
            let overall_ability = q_ability.get(validator_ability.3.unwrap().step_of).unwrap();
            (overall_ability.1.unwrap(), overall_ability.2.unwrap())
        };

        let holder_pos = if let Ok((_, p, _)) = q_transforms.get(holder_ent.0) {
            p.0
        } else {
            warn!("Holding entity not found");
            validator.value = false;
            continue;
        };

        let enemy_positions = q_transforms
            .iter()
            .filter_map(|(ent, pos, m_enemy)| m_enemy.map(|_| (ent, pos)))
            .collect::<Vec<(Entity, &Position)>>();
        let result = find_closest_in_list(1, holder_pos, &enemy_positions);
        if let Some((close_ent, dist)) = result.first() {
            if *dist <= range.0 {
                validator.value = true;
            } else {
                validator.value = false;
            }
        } else {
            validator.value = false;
        }
    }
}

#[derive(Component, Default, Debug, Clone, Copy)]
#[require(AbilityValidator = AbilityValidator::default())]
pub struct TargeterInAttackRange;

pub fn attack_range_targeter(
    mut q_validator: Query<(&mut AbilityValidator, &ValidatorOf), With<TargeterInAttackRange>>,
    q_ability: Query<(
        &Ability,
        Option<&AbilityOf>,
        Option<&AttackRange>,
        Option<&AbilityStep>,
    )>,
    q_holder: Query<&Position, Without<Targeter>>,
    q_targeter: Single<&Position, With<Targeter>>,
) {
    for (mut validator, v_of) in &mut q_validator {
        let validator_ability = q_ability.get(v_of.entity).expect("This should exist");
        let (holder_ent, range) = if validator_ability.3.is_none() {
            (validator_ability.1.unwrap(), validator_ability.2.unwrap())
        } else {
            let overall_ability = q_ability.get(validator_ability.3.unwrap().step_of).unwrap();
            (overall_ability.1.unwrap(), overall_ability.2.unwrap())
        };
        let holder_pos = q_holder
            .get(holder_ent.0)
            .expect("holder doesn't have a position?");
        validator.value = holder_pos.0.distance(q_targeter.0) <= range.0;
    }
}

#[derive(Component, Debug, Clone, Copy, FromTemplate)]
#[require(AbilityValidator = AbilityValidator::default())]
pub struct StepCompleted(pub Entity);
pub fn check_step_completed(
    mut q_validator: Query<(&mut AbilityValidator, &ValidatorOf, &StepCompleted)>,
    q_holder: Query<&HasAbilitySteps>,
    q_ability_state: Query<&AbilityState>,
) {
    for (mut validator, holder, check) in &mut q_validator {
        validator.value = false;
        if let Ok(steps) = q_holder.get(holder.entity) {
            if let Ok(state) = q_ability_state.get(check.0) {
                validator.value = matches!(*state, AbilityState::Completed);
            }
        }
    }
}
