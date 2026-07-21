//! This covers the damage popups
use bevy::prelude::*;
use lightyear::prelude::Controlled;

use crate::{
    shared::{
        combat::CombatEntity,
        damage::{DamageResult, DamageResultMessage},
        despawn_timer::DespawnTimer,
        enemies::Enemy,
        game_kinds::SinglePlayer,
        players::Player,
    },
    utils::CreatedBy,
};

pub struct PopupsRenderPlugin;

impl Plugin for PopupsRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_damage_popup, animate_damage_popup).chain());
    }
}

/// Floats over the head of the entity that's just been damaged
/// For now, the idea is that this will become a child of the entity.
/// I may want to revisit that
#[derive(Component)]
#[require(Name = "popup")]
pub struct DamagePopup;

fn spawn_damage_popup(
    mut commands: Commands,
    mut messages: MessageReader<DamageResultMessage>,
    q_controlling_player: Query<(), (With<Player>, Or<(With<Controlled>, With<SinglePlayer>)>)>,
    q_creator: Query<&CreatedBy>,
) {
    for m in messages.read() {
        let mut target = None;
        let entity_to_credit = if let Ok(e) = q_creator.get(m.damaging_entity) {
            e.0
        } else {
            m.damaging_entity
        };
        info!("Entity to credit: {:?}", entity_to_credit);
        if q_controlling_player.get(entity_to_credit).is_ok() {
            target = Some(m.damaged_entity)
        }
        if q_controlling_player.get(m.damaged_entity).is_ok() {
            target = Some(m.damaged_entity)
        }

        info!("Target to spawn {:?}", target);
        if target.is_none() {
            continue;
        }

        let text_color = if m.crit {
            Color::srgb(1.0, 0.0, 0.0)
        } else {
            Color::WHITE
        };

        match m.result {
            DamageResult::Apply(val) => {
                commands.entity(target.unwrap()).with_child((
                    DamagePopup,
                    Text2d::from(format!("{}", val)),
                    TextColor(text_color),
                    Transform::from_translation(Vec3::Y * 20.0),
                    CombatEntity,
                    DespawnTimer::new(0.5),
                ));
            }
            _ => {}
        }
    }
}

fn animate_damage_popup(mut q_popup: Query<(&mut Transform, &DespawnTimer), With<DamagePopup>>) {
    for (mut t, timer) in &mut q_popup {
        let pct = timer.0.fraction();
        // Want this to expand and contract as it moves up;
        let scale_factor = -8.0 * (pct - 0.5).powf(2.0) + 1.5;
        t.scale = Vec3::splat(scale_factor.clamp(0.2, 2.0));
        // Lerp in y dir for simplicity
        t.translation.y = 20.0 + pct * 20.0
    }
}
