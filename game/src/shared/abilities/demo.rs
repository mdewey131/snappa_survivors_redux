//! Code related to the setup of the gym, aka demo, area
use bevy::prelude::*;

use super::*;

pub fn targeting_step_ability_demo() -> impl SceneList {
    bsn_list! [(
        #DummyStepPlayer
        Player
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
                PassiveAbility
                HasValidators[
                    AbilityOffCooldown
                ]
                DrawTargeterOnMouse
            ),
            (
                #Step2
                AutoCast
                ActiveForTime(Timer::from_seconds(0.2, TimerMode::Once))
                RequestOnClick
                HasValidators [
                    TargeterInAttackRange
                ]
            )
            ]
        ]
    )
    ]
}
