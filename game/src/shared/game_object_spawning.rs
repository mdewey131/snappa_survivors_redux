//! Game Object Spawning
//!
//! "Game Object" in this context just refers to the entities that will be a part of
//! the actual gameplay (so, players, enemies, projectiles, weapons, pickups, etc)
//!
//! These entities share in common the idea that they need to either have lightyear
//! components like "Replicate" or "Predicted", or they need to have "SinglePlayer"
//! (collectively called "game kinds components")
//!
//! Because we want to have trigger systems that work off of the addition of the
//! game-relevant components, but we need the game kinds components for the filters,
//! the way to spawn these entities is to first add the game kinds components, then
//! to spawn the relevant bundle
use bevy::prelude::*;

use crate::{
    shared::{game_kinds::*, stats::RawStatsList},
    utils::AssetFolder,
};

/// Spawns the entity with the given bundle, ensuring that is happens in the order that is required
/// so that triggers can function correctly off of the DefaultClientFilter and DefaultServerFilter
pub fn spawn_game_object(
    commands: &mut Commands,
    game_kind: GameKinds,
    into_stats: Option<impl Into<AssetFolder>>,
    multiplayer_comp_options: MultiPlayerComponentOptions,
    bundle: impl Bundle,
) -> Entity {
    let entity = commands.spawn_empty().id();

    if let Some(stat_maker) = into_stats {
        let stats = RawStatsList::import_stats(stat_maker);
        stats.apply_to_character(entity, commands);
    }

    add_game_kinds_components(commands, entity, game_kind, multiplayer_comp_options);
    commands.entity(entity).insert(bundle);
    entity
}
