use bevy::{
    ecs::system::{BoxedSystem, SystemId},
    prelude::*,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "dev")]
use crate::utils::zoo::*;
use crate::{
    shared::{game_rules::GameRules, states::AppState, stats::xp::add_xp_manager},
    utils::SpawnPattern,
};

mod the_greens;
use the_greens::*;

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Reflect)]
#[reflect(Default)]
pub enum MapKind {
    #[default]
    TheGreens,
    #[cfg(feature = "dev")]
    DevZoo,
}

impl MapKind {
    fn character_spawner(&self, commands: &mut Commands) -> Box<SystemId> {
        let label = match self {
            MapKind::TheGreens => spawn_characters_the_greens,
            #[cfg(feature = "dev")]
            MapKind::DevZoo => spawn_zoo_characters,
        };
        let system_id = commands.register_system(label);
        Box::new(system_id)
    }
    fn interactables_spawner(&self, commands: &mut Commands) -> Box<SystemId> {
        let label = match self {
            MapKind::TheGreens => spawn_interactables_the_greens,
            #[cfg(feature = "dev")]
            MapKind::DevZoo => spawn_zoo_interactables,
        };
        Box::new(commands.register_system(label))
    }
    fn enemy_spawners(&self, commands: &mut Commands) -> Box<SystemId> {
        let label = match self {
            MapKind::TheGreens => enemy_spawners_the_greens,
            #[cfg(feature = "dev")]
            MapKind::DevZoo => spawn_zoo_enemies,
        };
        Box::new(commands.register_system(label))
    }

    fn map_elements(&self, commands: &mut Commands) -> Box<SystemId> {
        let label = match self {
            MapKind::TheGreens => map_elements_the_greens,
            #[cfg(feature = "dev")]
            MapKind::DevZoo => spawn_zoo_interactables,
        };
        Box::new(commands.register_system(label))
    }
    fn custom_systems(&self, commands: &mut Commands) -> Vec<Box<SystemId>> {
        let mut ret = Vec::new();
        match self {
            #[cfg(feature = "dev")]
            MapKind::DevZoo => ret.push(commands.register_system(spawn_zoo_weapons)),
            _ => {
                ret.push(Box::new(commands.register_system(add_xp_manager)));
            }
        }
        ret
    }
}

/// A resource that tracks which systems are responsible for loading the map.
/// These are completely generic sytems because we expect that maps will want
/// their own way of defining these common functions, on top of potentially
/// having more things to do based on the initial sytems
///
/// For the Client, in multiplayer scenarios, this is not run, but it is in
/// single player scenarios.
#[derive(Resource, Debug)]
pub struct MapLoadingSystems {
    pub characters: Box<SystemId>,
    pub interactables: Box<SystemId>,
    pub enemy_spawners: Box<SystemId>,
    pub map_elements: Box<SystemId>,
    pub custom_systems: Vec<Box<SystemId>>,
}

pub fn add_map_loading_systems(mut commands: Commands, game_rules: Res<GameRules>) {
    let characters = game_rules.map_type.character_spawner(&mut commands);
    let interactables = game_rules.map_type.interactables_spawner(&mut commands);
    let enemy_spawners = game_rules.map_type.enemy_spawners(&mut commands);
    let map_elements = game_rules.map_type.map_elements(&mut commands);
    let custom_systems = game_rules.map_type.custom_systems(&mut commands);

    commands.insert_resource(MapLoadingSystems {
        characters,
        interactables,
        enemy_spawners,
        map_elements,
        custom_systems,
    });
}

pub fn run_map_loading_systems(mut commands: Commands, loading_systems: Res<MapLoadingSystems>) {
    commands.run_system(*loading_systems.map_elements);
    commands.run_system(*loading_systems.enemy_spawners);
    commands.run_system(*loading_systems.interactables);
    commands.run_system(*loading_systems.characters);
    for sys in loading_systems.custom_systems.iter() {
        commands.run_system(**sys)
    }
}
