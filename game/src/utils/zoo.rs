use crate::shared::{
    combat::CharacterFacing,
    game_kinds::{CurrentGameKind, GameKinds, MultiPlayerComponentOptions},
    game_object_spawning::{SpawnGameObject, spawn_game_object},
    game_rules::GameRules,
    maps::*,
    players::{CharacterKind, Player, PlayerBaseBundle, PlayerWeapons},
    states::AppState,
    stats::RawStatsList,
    upgrades::PlayerUpgradeSlots,
    weapons::{WeaponKind, add_weapon_to_character},
};
use avian2d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::PeerId;
use strum::IntoEnumIterator;

const DISPLAY_PEN_COLS: usize = 2;

const CHARACTER_DISPLAY_PEN_WIDTH: f32 = 300.0;
const CHARACTER_DISPLAY_PEN_HEIGHT: f32 = 300.0;
const CHARACTER_DISPLAY_GROUP_CENTERPOINT: (f32, f32) = (-600.0, 0.0);

const WEAPON_DISPLAY_PEN_WIDTH: f32 = 300.0;
const WEAPON_DISPLAY_PEN_HEIGHT: f32 = 300.0;
const WEAPON_DISPLAY_GROUP_CENTERPOINT: (f32, f32) = (600.0, 0.0);

#[derive(Component)]
pub struct ZooPen {
    pub width: f32,
    pub height: f32,
    pub name: String,
    pub color: Color,
}

impl ZooPen {
    fn new(width: f32, height: f32, name: String, color: Color) -> Self {
        Self {
            width,
            height,
            name,
            color,
        }
    }
}

#[derive(Component)]
#[require(ZooPen = character_pen())]
pub struct CharacterDisplayGroup;
fn character_pen() -> ZooPen {
    let len = CharacterKind::iter().len();
    let num_rows = (len / DISPLAY_PEN_COLS);
    let total_w = WEAPON_DISPLAY_PEN_HEIGHT * num_rows as f32;
    let total_h = WEAPON_DISPLAY_PEN_WIDTH * DISPLAY_PEN_COLS as f32;
    ZooPen::new(
        total_w,
        total_h,
        "Characters".into(),
        Color::srgb(1.0, 0.0, 0.0),
    )
}

#[derive(Component)]
#[require(ZooPen = weapon_pen())]
pub struct WeaponDisplayGroup;
fn weapon_pen() -> ZooPen {
    let len = WeaponKind::iter().len();
    let num_rows = (len / DISPLAY_PEN_COLS);
    let total_w = WEAPON_DISPLAY_PEN_HEIGHT * num_rows as f32;
    let total_h = WEAPON_DISPLAY_PEN_WIDTH * DISPLAY_PEN_COLS as f32;
    ZooPen::new(
        total_w,
        total_h,
        "Weapons".into(),
        Color::srgb(0.0, 0.0, 1.0),
    )
}

#[cfg(feature = "dev")]
pub struct ZooLevelPlugin;

#[cfg(feature = "dev")]
impl Plugin for ZooLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_zoo_gizmos);
    }
}

pub fn launch_zoo_level(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    mut game_kind: ResMut<CurrentGameKind>,
) {
    info!("Transitioning to zoo level");
    game_kind.0 = Some(GameKinds::SinglePlayer);
    let mut game_rules = GameRules::default();
    game_rules.map_type = crate::shared::maps::MapKind::DevZoo;
    commands.insert_resource(game_rules);
    state.set(AppState::LoadingLevel);
}

pub fn spawn_zoo_characters(mut commands: Commands) {
    info!("Spawning Characters");
    let iter = CharacterKind::iter();
    let len = iter.len();
    // 1 for padding, may not be necessary
    let num_rows = (len / DISPLAY_PEN_COLS);
    for (i, char) in iter.enumerate() {
        let col_pos = (i % DISPLAY_PEN_COLS);
        let row_pos = (i * num_rows) / len;
        info!("Found col_pos: {}, row_pos: {}", col_pos, row_pos);
        let total_width = CHARACTER_DISPLAY_PEN_WIDTH * num_rows as f32;
        let total_height = CHARACTER_DISPLAY_PEN_HEIGHT * DISPLAY_PEN_COLS as f32;
        let pos_x = ((row_pos as f32 * CHARACTER_DISPLAY_PEN_WIDTH)
            + (CHARACTER_DISPLAY_PEN_WIDTH / 2.0))
            - (total_width / 2.0)
            + CHARACTER_DISPLAY_GROUP_CENTERPOINT.0;
        let pos_y = ((col_pos as f32 * CHARACTER_DISPLAY_PEN_HEIGHT)
            + (CHARACTER_DISPLAY_PEN_HEIGHT / 2.0))
            - (total_height / 2.0)
            + CHARACTER_DISPLAY_GROUP_CENTERPOINT.1;

        let player = Player {
            client: None,
            character: char,
        };
        let player_stats = RawStatsList::import_stats(player.character);
        let player_ent = commands
            .spawn((
                PlayerBaseBundle {
                    player,
                    position: Position(Vec2::new(pos_x, pos_y)),
                    upgrade_slots: PlayerUpgradeSlots::new(5, 5),
                    weapons: PlayerWeapons::default(),
                    facing: CharacterFacing::default(),
                },
                DespawnOnExit(AppState::InGame),
            ))
            .id();

        player_stats.apply_to_character(player_ent, &mut commands);

        let text = commands
            .spawn((
                Text2d::new(format!("{:?}", player.character)),
                Transform::from_translation(Vec3::NEG_Y * 20.0),
            ))
            .id();
        commands.entity(player_ent).add_child(text);
    }

    commands.spawn((
        CharacterDisplayGroup,
        DespawnOnExit(AppState::InGame),
        Transform::from_translation(Vec3::new(
            CHARACTER_DISPLAY_GROUP_CENTERPOINT.0,
            CHARACTER_DISPLAY_GROUP_CENTERPOINT.1,
            CHARACTER_DISPLAY_GROUP_CENTERPOINT.1,
        )),
    ));
}

pub fn spawn_zoo_weapons(mut commands: Commands) {
    let _span = info_span!("Spawning weapons").entered();

    let weapons_iter = WeaponKind::iter();
    let len = weapons_iter.len();
    let num_rows = (len / DISPLAY_PEN_COLS);

    for (i, weapon) in weapons_iter.enumerate() {
        let col_pos = (i % DISPLAY_PEN_COLS);
        let row_pos = (i * num_rows) / len;
        let total_width = WEAPON_DISPLAY_PEN_WIDTH * num_rows as f32;
        let total_height = WEAPON_DISPLAY_PEN_HEIGHT * DISPLAY_PEN_COLS as f32;
        let pos_x = ((row_pos as f32 * WEAPON_DISPLAY_PEN_WIDTH)
            + (WEAPON_DISPLAY_PEN_WIDTH / 2.0))
            - (total_width / 2.0)
            + WEAPON_DISPLAY_GROUP_CENTERPOINT.0;
        let pos_y = ((col_pos as f32 * WEAPON_DISPLAY_PEN_HEIGHT)
            + (WEAPON_DISPLAY_PEN_HEIGHT / 2.0))
            - (total_height / 2.0)
            + WEAPON_DISPLAY_GROUP_CENTERPOINT.1;

        // Always spawn dewey, he has stats that we can use as a baseline
        let player = Player {
            client: None,
            character: CharacterKind::Dewey,
        };
        let player_stats = RawStatsList::import_stats(player.character);
        let player_ent = commands
            .spawn((
                PlayerBaseBundle {
                    player,
                    position: Position(Vec2::new(pos_x, pos_y)),
                    upgrade_slots: PlayerUpgradeSlots::new(5, 5),
                    weapons: PlayerWeapons::default(),
                    facing: CharacterFacing::default(),
                },
                DespawnOnExit(AppState::InGame),
            ))
            .id();

        player_stats.apply_to_character(player_ent, &mut commands);

        add_weapon_to_character(player_ent, weapon, &mut commands, GameKinds::SinglePlayer);
    }
    commands.spawn((
        WeaponDisplayGroup,
        DespawnOnExit(AppState::InGame),
        Transform::from_translation(Vec3::new(
            WEAPON_DISPLAY_GROUP_CENTERPOINT.0,
            WEAPON_DISPLAY_GROUP_CENTERPOINT.1,
            WEAPON_DISPLAY_GROUP_CENTERPOINT.1,
        )),
    ));
}

/// TODO
pub fn spawn_zoo_interactables() {}

/// TODO
pub fn spawn_zoo_enemies() {}

fn draw_zoo_gizmos(mut gizmos: Gizmos, q_pens: Query<(&Transform, &ZooPen)>) {
    for (pos, pen) in &q_pens {
        gizmos.rect_2d(
            Isometry2d::from_translation(pos.translation.truncate()),
            Vec2::new(pen.width, pen.height),
            pen.color,
        )
    }
}
