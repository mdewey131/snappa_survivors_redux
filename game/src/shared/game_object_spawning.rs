use bevy::prelude::*;

use crate::{
    shared::{game_kinds::*, states::*, stats::RawStatsList},
    utils::AssetFolder,
};

/// Game Object Spawning
///
/// "Game Object" in this context just refers to the entities that will be a part of
/// the actual gameplay (so, players, enemies, projectiles, weapons, pickups, etc)
///
/// These entities share in common the idea that they need to either have lightyear
/// components like "Replicate" or "Predicted", or they need to have "SinglePlayer")
///
/// Spawns the entity with the given bundle, ensuring that is happens in the required order.
///
/// Because we want to have trigger systems that work off of the addition of the
/// game-relevant components (e.g. stats), but we need the game kinds components for the filters to those triggers,
/// the way to spawn these entities is to first add the game kinds components, then
/// to spawn the relevant bundle
///
///
/// Inserting the stats on the entity as it spawns is important for other spawn triggers that run on this
/// entity which require the stats, which we'll miss if we delay inserting the stats.
///
/// The same is true for replication components, so we take those settings as well
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
    commands
        .entity(entity)
        .insert((bundle, DespawnOnExit(AppState::InGame)));
    entity
}

/// Trying this in a slightly different approach - Looking to make a custom command
pub struct SpawnGameObject<B: Bundle> {
    bundle: B,
    stats: Option<AssetFolder>,
    multiplayer_options: MultiPlayerComponentOptions,
}

impl<B: Bundle> SpawnGameObject<B> {
    pub fn new(options: MultiPlayerComponentOptions, bundle: B) -> Self {
        Self {
            bundle,
            stats: None,
            multiplayer_options: options,
        }
    }

    pub fn with_stats(&mut self, s: impl Into<AssetFolder>) -> &mut Self {
        let asset_folder = s.into();
        self.stats = Some(asset_folder);
        self
    }
}

impl<B: Bundle> Command for SpawnGameObject<B> {
    fn apply(self, world: &mut World) -> () {
        let entity = world.spawn_empty().id();

        if let Some(stat_path) = self.stats {
            let stats = RawStatsList::import_stats(stat_path);
            stats.apply_to_character(entity, &mut world.commands());
        }

        let game_kind = world
            .resource::<CurrentGameKind>()
            .0
            .expect("You should have a current game kind whenever you're spawning a game object");

        add_game_kinds_components(
            &mut world.commands(),
            entity,
            game_kind,
            self.multiplayer_options,
        );

        world
            .commands()
            .entity(entity)
            .insert((self.bundle, DespawnOnExit(AppState::InGame)));
    }
}
