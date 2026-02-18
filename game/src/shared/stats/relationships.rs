use super::*;
use crate::shared::weapons::Weapon;
use bevy::prelude::*;

/// In the future, this can do some really interesting stuff in order for us to control relationship
/// rules at a high level of granularity. For now, this is dead simple
pub struct StatRelationshipsPlugin;

impl Plugin for StatRelationshipsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(relate_weapon_stats_to_player);
    }
}

fn relate_weapon_stats_to_player(
    trigger: On<Add, Weapon>,
    mut q_weapon: Query<(&mut StatList, &ChildOf), With<Weapon>>,
    q_player: Query<&StatList, Without<ChildOf>>,
) {
    // Every weapon stats relates to the player
    //
    if let Ok((mut weapon_stats, child)) = q_weapon.get_mut(trigger.entity) {
        let player_stats = q_player.get(child.0).expect("Player Stats not found!");
        let mut todo = Vec::new();
        for stat_kind in weapon_stats.list.keys() {
            let sk = *stat_kind;
            let player_copy = if let Some(pc) = player_stats.list.get(&sk) {
                pc
            } else {
                continue;
            };
            todo.push((sk, player_copy.clone_current_weak()))
        }
        for (sk, handle) in todo.drain(..) {
            if let Some(ref mut weapon_stat) = weapon_stats.list.get_mut(&sk) {
                let modifier = match sk {
                    StatKind::ProjBounces | StatKind::ProjCount => StatModifierMethod::FlatAdd,
                    _ => StatModifierMethod::MultipliyWithBase { coefficient: 1.0 },
                };
                let modifier = StatModifier::new(handle, modifier);
                weapon_stat.modifiers.push(modifier);
            }
        }
    }
}
