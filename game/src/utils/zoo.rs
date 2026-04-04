use crate::shared::{
    combat::CharacterFacing,
    game_kinds::{CurrentGameKind, GameKinds},
    game_object_spawning::SpawnGameObject,
    game_rules::GameRules,
    players::{CharacterKind, Player, PlayerBaseBundle, PlayerWeapons},
    states::AppState,
    upgrades::PlayerUpgradeSlots,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::PeerId;
use strum::IntoEnumIterator;

const CHARACTER_DISPLAY_PEN_WIDTH: f32 = 50.0;
const CHARACTER_DISPLAY_PEN_HEIGHT: f32 = 50.0;

pub fn launch_zoo_level(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    mut game_kind: ResMut<CurrentGameKind>,
) {
    info!("Transitioning to zoo level");
    game_kind.0 = Some(GameKinds::SinglePlayer);
    let mut game_rules = GameRules::default();
    game_rules.map_type = crate::shared::game_rules::MapKind::DevZoo;
    commands.insert_resource(game_rules);
    state.set(AppState::LoadingLevel);
}

pub fn spawn_zoo_characters(mut commands: Commands) {
    info!("Spawning Characters");
    let iter = CharacterKind::iter();
    let len = iter.len();
    for (i, char) in iter.enumerate() {
        info!("Indiviudual loop");
        let num_rows = 4;
        // 1 for padding, may not be necessary
        let num_cols = (len / num_rows) + 1;
        let row_pos = i % num_rows;
        let col_pos = (i / num_cols);
        info!("Found col_pos: {}, row_pos: {}", col_pos, row_pos);
        let total_width = CHARACTER_DISPLAY_PEN_WIDTH * num_rows as f32;
        let total_height = CHARACTER_DISPLAY_PEN_HEIGHT * num_cols as f32;
        let pos_x = ((row_pos as f32 * CHARACTER_DISPLAY_PEN_WIDTH)
            + (CHARACTER_DISPLAY_PEN_WIDTH / 2.0))
            - (total_width / 2.0);
        let pos_y = ((col_pos as f32 * CHARACTER_DISPLAY_PEN_HEIGHT)
            + (CHARACTER_DISPLAY_PEN_HEIGHT / 2.0))
            - (total_height / 2.0);

        let player = Player {
            client: PeerId::Local(0),
            character: char,
        };

        commands.queue(SpawnGameObject::new(
            crate::shared::game_kinds::MultiPlayerComponentOptions::PREDICTED,
            (PlayerBaseBundle {
                player,
                position: Position(Vec2::new(pos_x, pos_y)),
                upgrade_slots: PlayerUpgradeSlots::new(5, 5),
                weapons: PlayerWeapons::default(),
                facing: CharacterFacing::default(),
            }),
        ));
    }
}
