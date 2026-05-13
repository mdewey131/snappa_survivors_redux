use avian2d::prelude::*;
use bevy::{
    ecs::system::{BoxedSystem, SystemId},
    prelude::*,
};
use bluenoise::BlueNoise;
use lightyear::core::id::RemoteId;
use rand::{SeedableRng, rngs::SmallRng};
use serde::{Deserialize, Serialize};

#[cfg(feature = "dev")]
use crate::utils::zoo::*;
use crate::{
    render::map::{ChunkOf, MapBackground, MapChunk},
    shared::{
        enemies::spawner::add_enemy_spawner,
        game_kinds::CurrentGameKind,
        game_rules::GameRules,
        interactables::{
            BeerShrine, BeerShrineChargeRadius, beer_shrine_collider,
            beer_shrine_collider_detection_range,
        },
        lobby::PlayerInLobby,
        players::spawn_characters,
        states::AppState,
        stats::xp::add_xp_manager,
    },
    utils::SpawnPattern,
};

mod the_greens;
use the_greens::*;

const MAP_BUILDER_SEED: u64 = 0;

#[derive(Clone, Serialize, Deserialize)]
pub struct MapBuilderSettings {
    pub map_size_tiles: (u32, u32),
    pub tile_size: Vec2,
    pub num_health_spawners: u32,
    pub num_beer_shrines: u32,
    pub space_between_interactables: f32,
    pub character_spawning_center: Vec2,
    pub character_spawning_radius: f32,
    pub audio_path: String,
    pub enemy_spawner_path: String,
}

#[derive(Resource)]
pub struct MapBuilder {
    pub noise: BlueNoise<SmallRng>,
    pub settings: MapBuilderSettings,
}

impl MapBuilder {
    fn new(map_kind: MapKind) -> Self {
        let path = match map_kind {
            MapKind::TheGreens => "grass/settings.ron",
            _ => {
                unimplemented!()
            }
        };

        let settings: MapBuilderSettings = crate::utils::read_ron(format!("assets/maps/{}", path));

        settings.into()
    }
}

impl From<MapBuilderSettings> for MapBuilder {
    fn from(value: MapBuilderSettings) -> Self {
        let total_elements_to_spawn = value.num_beer_shrines + value.num_health_spawners;
        let total_size_x = value.map_size_tiles.0 as f32 * value.tile_size.x;
        let total_size_y = value.map_size_tiles.1 as f32 * value.tile_size.y;

        let mut noise = BlueNoise::<SmallRng>::new(
            total_size_x,
            total_size_y,
            value.space_between_interactables,
        );

        noise
            .with_seed(MAP_BUILDER_SEED)
            .with_samples(total_elements_to_spawn);

        Self {
            noise,
            settings: value,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Reflect)]
#[reflect(Default)]
pub enum MapKind {
    #[default]
    TheGreens,
    #[cfg(feature = "dev")]
    DevZoo,
}

impl MapKind {
    fn map_background(&self, commands: &mut Commands) -> Box<SystemId> {
        match self {
            #[cfg(feature = "dev")]
            MapKind::DevZoo => Box::new(commands.register_system(map_chunks_the_greens)),
            _ => Box::new(commands.register_system(spawn_map_chunks)),
        }
    }

    fn character_spawner(&self, commands: &mut Commands) -> Box<SystemId> {
        match self {
            #[cfg(feature = "dev")]
            MapKind::DevZoo => Box::new(commands.register_system(spawn_zoo_characters)),
            _ => Box::new(commands.register_system(spawn_characters_in_map)),
        }
    }
    fn interactables_spawner(&self, commands: &mut Commands) -> Box<SystemId> {
        match self {
            #[cfg(feature = "dev")]
            MapKind::DevZoo => Box::new(commands.register_system(spawn_zoo_interactables)),
            MapKind::TheGreens => Box::new(commands.register_system(spawn_interactables_in_map)),
        }
    }
    fn enemy_spawners(&self, commands: &mut Commands) -> Box<SystemId> {
        match self {
            MapKind::TheGreens => Box::new(commands.register_system(add_enemy_spawner)),
            #[cfg(feature = "dev")]
            MapKind::DevZoo => Box::new(commands.register_system(spawn_zoo_enemies)),
        }
    }

    fn map_elements(&self, commands: &mut Commands) -> Box<SystemId> {
        match self {
            MapKind::TheGreens => Box::new(commands.register_system(map_elements_the_greens)),
            #[cfg(feature = "dev")]
            MapKind::DevZoo => Box::new(commands.register_system(spawn_zoo_interactables)),
        }
    }
    fn custom_systems(&self, commands: &mut Commands) -> Vec<Box<SystemId>> {
        let mut ret = Vec::new();
        match self {
            #[cfg(feature = "dev")]
            MapKind::DevZoo => ret.push(Box::new(commands.register_system((spawn_zoo_weapons)))),
            _ => {}
        }
        ret
    }
}

/// A resource that tracks which systems are responsible for placing things around the maps.
/// These are completely generic sytems because we expect that maps will want
/// their own way of defining these common functions, on top of potentially
/// having more things to do based on the initial sytems
///
/// For the Client, in multiplayer scenarios, this is not run, but it is in
/// single player scenarios.
#[derive(Resource, Debug)]
pub struct MapBuilderSystems {
    pub map_background: Box<SystemId>,
    pub characters: Box<SystemId>,
    pub interactables: Box<SystemId>,
    pub enemy_spawners: Box<SystemId>,
    pub map_elements: Box<SystemId>,
    pub custom_systems: Vec<Box<SystemId>>,
}

pub fn initialiize_map_builder(mut commands: Commands, game_rules: Res<GameRules>) {
    let map_builder = MapBuilder::new(game_rules.map_type);
    commands.insert_resource(map_builder);
}

pub fn add_map_loading_systems(mut commands: Commands, game_rules: Res<GameRules>) {
    let map_background = game_rules.map_type.map_background(&mut commands);
    let characters = game_rules.map_type.character_spawner(&mut commands);
    let interactables = game_rules.map_type.interactables_spawner(&mut commands);
    let enemy_spawners = game_rules.map_type.enemy_spawners(&mut commands);
    let map_elements = game_rules.map_type.map_elements(&mut commands);
    let custom_systems = game_rules.map_type.custom_systems(&mut commands);

    commands.insert_resource(MapBuilderSystems {
        map_background,
        characters,
        interactables,
        enemy_spawners,
        map_elements,
        custom_systems,
    });
}

pub fn run_map_loading_systems(mut commands: Commands, loading_systems: Res<MapBuilderSystems>) {
    commands.run_system(*loading_systems.map_background);
    commands.run_system(*loading_systems.map_elements);
    commands.run_system(*loading_systems.enemy_spawners);
    commands.run_system(*loading_systems.interactables);
    commands.run_system(*loading_systems.characters);
    for sys in loading_systems.custom_systems.iter() {
        commands.run_system(**sys)
    }
}

fn spawn_map_chunks(mut commands: Commands, builder: Res<MapBuilder>) {
    // Making the map somewhat huge to start
    let texture_size = Vec2::new(128.0, 128.0);
    let map = commands
        .spawn((
            MapBackground,
            Transform::default(),
            DespawnOnExit(AppState::InGame),
        ))
        .id();

    let total_size_x = builder.settings.map_size_tiles.0 as f32 * builder.settings.tile_size.x;
    let total_size_y = builder.settings.map_size_tiles.1 as f32 * builder.settings.tile_size.y;
    for x in (0..builder.settings.map_size_tiles.0 as usize) {
        for y in (0..builder.settings.map_size_tiles.1 as usize) {
            commands.spawn((
                Transform::from_translation(Vec3::new(
                    x as f32 * texture_size.x - (total_size_x / 2.0),
                    y as f32 * texture_size.y - (total_size_y / 2.0),
                    -1000.0,
                )),
                ChunkOf(map),
                MapChunk,
                // this is for bevy inspector egui reasons
                ChildOf(map),
            ));
        }
    }
}

pub fn spawn_characters_in_map(
    mut commands: Commands,
    builder: Res<MapBuilder>,
    game_kinds: Res<CurrentGameKind>,
    q_player: Query<(Entity, &PlayerInLobby, Option<&RemoteId>)>,
) {
    let n_char = q_player.iter().len();
    let mut spawn_pos = SpawnPattern::Circle {
        amount: n_char as u8,
        center: builder.settings.character_spawning_center,
        radius: builder.settings.character_spawning_radius,
        radius_only: true,
    }
    .to_positions();

    spawn_characters(&mut spawn_pos, &mut commands, &game_kinds, &q_player);
}

pub fn spawn_interactables_in_map(mut commands: Commands, mut builder: ResMut<MapBuilder>) {
    let total_size_x = builder.settings.map_size_tiles.0 as f32 * builder.settings.tile_size.x;
    let total_size_y = builder.settings.map_size_tiles.1 as f32 * builder.settings.tile_size.y;
    info!("Spawning Points!");
    let num_to_take = builder.settings.num_beer_shrines as usize;
    let noise = &mut builder.noise;
    let points_to_spawn = noise.take(num_to_take);
    for point in points_to_spawn {
        let position = Vec2::new(
            point.x - (total_size_x / 2.0),
            point.y - (total_size_y / 2.0),
        );
        commands
            .spawn((
                BeerShrine {
                    max_charge: 10.0,
                    current_charge: 10.0,
                    charge_rate_secs: 0.75,
                },
                Position(position),
                beer_shrine_collider(),
            ))
            .with_child((
                BeerShrineChargeRadius,
                beer_shrine_collider_detection_range(),
                Sensor,
            ));
    }
}
