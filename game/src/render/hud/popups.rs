//! This covers the damage popups
use bevy::prelude::*;
use lightyear::prelude::Controlled;

use crate::{
    shared::{
        combat::CombatEntity,
        damage::{HealthChange, HealthChangeMessage, HealthChangeResult},
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
        app.add_systems(Update, (spawn_popup, animate_damage_popup).chain());
    }
}

/// Floats over the head of the entity that's just been damaged
/// For now, the idea is that this will become a child of the entity.
/// I may want to revisit that
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(Name = "popup")]
pub struct DamagePopup;

fn health_change_popup(
    on: Entity,
    kind: HealthChange,
    res: HealthChangeResult,
    amount: f32,
) -> impl Scene {
    let (text_color, text) = match res {
        HealthChangeResult::Normal => {
            let formatted = format!("{:.1}", amount);
            match kind {
                HealthChange::Heal => (Color::srgba(0.8, 0.8, 0.4, 1.0), formatted),
                HealthChange::Damage => (Color::WHITE, formatted),
            }
        }
        HealthChangeResult::Crit => {
            let formatted = format!("{:.1}", amount);
            match kind {
                HealthChange::Heal => (Color::srgba(1.0, 1.0, 0.4, 1.0), formatted),
                HealthChange::Damage => (Color::srgb(1.0, 0.0, 0.0), formatted),
            }
        }
        HealthChangeResult::Evaded => (Color::srgba(0.1, 0.1, 0.9, 1.0), "Evaded!".into()),
        HealthChangeResult::Invulnerable => (Color::WHITE, "Invulnerable!".into()),
        _ => unimplemented!(),
    };

    bsn! {
        ChildOf(on)
        DamagePopup
        Text2d::from(text)
        TextColor(text_color)
        Transform::from_translation(Vec3::Y * 20.0)
        CombatEntity
        DespawnTimer::new(0.5)
    }
}

fn spawn_popup(
    mut commands: Commands,
    mut messages: MessageReader<HealthChangeMessage>,
    q_controlling_player: Query<(), (With<Player>, Or<(With<Controlled>, With<SinglePlayer>)>)>,
    q_creator: Query<&CreatedBy>,
) {
    for m in messages.read() {
        let mut target = None;
        let entity_to_credit = if let Ok(e) = q_creator.get(m.source_entity) {
            e.0
        } else {
            m.source_entity
        };
        info!("Entity to credit: {:?}", entity_to_credit);
        if q_controlling_player.get(entity_to_credit).is_ok() {
            target = Some(m.receiving_entity)
        }
        if q_controlling_player.get(m.receiving_entity).is_ok() {
            target = Some(m.receiving_entity)
        }
        info!("Target to spawn {:?}", target);
        if target.is_none() {
            continue;
        }

        match m.result {
            HealthChangeResult::EntityAlreadyDead | HealthChangeResult::DidNothing => {}
            _ => {
                commands.spawn_scene(health_change_popup(
                    target.unwrap(),
                    m.kind,
                    m.result,
                    m.amount,
                ));
            }
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
