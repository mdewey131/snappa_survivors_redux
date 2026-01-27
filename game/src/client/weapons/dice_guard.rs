use bevy::prelude::*;

use crate::{
    render::weapons::add_dice_guard_rendering_components,
    shared::{
        game_kinds::DefaultClientFilter,
        stats::{StatKind, StatList, StatModifier, StatModifierMethod},
        weapons::*,
    },
};

pub struct ClientDiceGuardPlugin;
impl Plugin for ClientDiceGuardPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(dice_guard_activate::<DefaultClientFilter>)
            .add_observer(dice_guard_deactivate::<DefaultClientFilter>)
            .add_observer(_tmp_add_stat_relationship_components);
    }
}

pub struct ClientDiceGuardRenderPlugin;
impl Plugin for ClientDiceGuardRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(add_dice_guard_rendering_components::<DefaultClientFilter>);
    }
}

fn _tmp_add_stat_relationship_components(
    trigger: On<Add, Weapon>,
    mut q_weapon: Query<(&ChildOf, &mut StatList)>,
    q_player: Query<&StatList, Without<ChildOf>>,
) {
    if let Ok((child, mut weapon_stats)) = q_weapon.get_mut(trigger.entity) {
        if let Ok(player_stats) = q_player.get(child.0) {
            let player_dam = if let Some(pd) = player_stats.list.get(&StatKind::Damage) {
                pd
            } else {
                return;
            };
            if let Some(ref mut dam_stat) = weapon_stats.list.get_mut(&StatKind::Damage) {
                dam_stat.modifiers.push(StatModifier::new(
                    player_dam.clone_current_weak(),
                    StatModifierMethod::MultipliyWithBase { coefficient: 1.0 },
                ));
            }
        }
    }
}
