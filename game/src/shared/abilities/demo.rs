//! Code related to the setup of the gym, aka demo, area
use crate::shared::enemies::spawner::EnemySpawnPattern;
use bevy::prelude::*;

use super::*;

pub fn targeting_step_ability_demo() -> impl SceneList {
    bsn_list! [(
        #DummyStepPlayer
        Player
        Position(Vec2::ZERO)
        HasAbilities [
            #TargetedStepAbility
            Ability
            AddCooldownOnCompletion
            AttackRange(100.0)
            Damage(5.0)
            CooldownRate(5.0)
            HasAbilitySteps [
            (
                #Step1
                RequestOnInput(String::from("E"))
                HasValidators[
                    AbilityOffCooldown
                ]
                DrawTargeterOnMouse
                DrawAttackRangeRadius
            ),
            (
                #Step2
                CompletesInstantly
                RequestOnClick
                SpawnEnemies(EnemySpawnInstruction {
                    kind: EnemyKind::FacelessMan,
                    pattern: EnemySpawnPattern::SingleLocation(Vec2::new(100.0, 0.0))
                })
                HasValidators [
                    TargeterInAttackRange
                ]

            )
            ]
        ]
    )
    ]
}
