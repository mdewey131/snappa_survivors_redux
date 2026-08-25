//! Code related to the setup of the gym, aka demo, area
use crate::shared::enemies::spawner::EnemySpawnPattern;
use bevy::prelude::*;

use super::*;

pub fn dice_guard_demo(position: Vec2) -> impl SceneList {
    let enemy_pos = position + Vec2::X * 50.0;
    bsn_list! [(
        #DGPlayer
        Player
        Position(position)
        HasAbilities [
            dice_guard()
        ]
    ),
    (
        #Enemy
        Enemy
        Sprite {image: "enemies/faceless/sprite.png" }
        Position(enemy_pos)
    )]
}

pub fn bump_tunes_demo(position: Vec2) -> impl SceneList {
    let e_pos_1 = position + Vec2::X * 100.0;
    let e_pos_2 = position + Vec2::X * -100.0;
    let e_pos_3 = position + Vec2::Y * 100.0;
    let e_pos_4 = position + Vec2::Y * -100.0;
    bsn_list! [(
            #TunesPlayer
            Player
            Position(position)
            HasAbilities [
                bump_tunes::<Enemy>()
            ]
        ),
        (
            #TunesEnemy1
            Enemy
            Health::new(50.0)
            Sprite {image: "enemies/faceless/sprite.png" }
            Position(e_pos_1)
        ),
        (
            #TunesEnemy2
            Enemy
            Health::new(50.0)
            Sprite {image: "enemies/faceless/sprite.png" }
            Position(e_pos_2)
        ),
        (
            #TunesEnemy3
            Enemy
            Health::new(50.0)
            Sprite {image: "enemies/faceless/sprite.png" }
            Position(e_pos_3)
        ),
        (
            #TunesEnemy4
            Enemy
            Health::new(50.0)
            Sprite {image: "enemies/faceless/sprite.png" }
            Position(e_pos_4)
        )
    ]
}

pub fn targeting_step_ability_demo(position: Vec2) -> impl SceneList {
    bsn_list! [(
        #DummyStepPlayer
        Player
        Position(position)
        HasAbilities [
            #TargetedStepAbility
            Ability
            AddCooldownOnCompletion
            AttackRange(300.0)
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

pub fn throw_hands_demo(position: Vec2) -> impl SceneList {
    let e_pos_1 = position + Vec2::X * 100.0;
    let e_pos_2 = position + Vec2::X * -100.0;
    let e_pos_3 = position + Vec2::Y * 100.0;
    let e_pos_4 = position + Vec2::Y * -100.0;
    bsn_list! [(
            #HandsPlayer
            Player
            Position(position)
            HasAbilities [
                throw_hands::<Enemy>()
            ]
        ),
        (
            #HandsEnemy1
            Enemy
            Health::new(50.0)
            Sprite {image: "enemies/faceless/sprite.png" }
            Position(e_pos_1)
        ),
        (
            #HandsEnemy2
            Enemy
            Health::new(50.0)
            Sprite {image: "enemies/faceless/sprite.png" }
            Position(e_pos_2)
        ),
        (
            #HandsEnemy3
            Enemy
            Health::new(50.0)
            Sprite {image: "enemies/faceless/sprite.png" }
            Position(e_pos_3)
        ),
        (
            #HandsEnemy4
            Enemy
            Health::new(50.0)
            Sprite {image: "enemies/faceless/sprite.png" }
            Position(e_pos_4)
        )
    ]
}
