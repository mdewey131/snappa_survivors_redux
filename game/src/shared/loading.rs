use crate::shared::{
    combat::CharacterFacing, game_kinds::*, game_object_spawning::*, lobby::PlayerInLobby,
    players::*, states::AppState, upgrades::PlayerUpgradeSlots, weapons::*,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::*;
use rand::Rng;

const EXPECTED_LOADING_CONFIRMATION_FRAMES: u8 = 30;

/// The set of common functions that are needed by both server and client.
///
/// This is primarily responsible to keep track of what's available in `LoadingAssets`
pub struct SharedLoadingPlugin;
impl Plugin for SharedLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LevelLoadingState>()
            .insert_resource(LoadingAssets::new(EXPECTED_LOADING_CONFIRMATION_FRAMES))
            .add_systems(
                Update,
                check_loading_assets.run_if(in_state(AppState::LoadingLevel)),
            );
    }
}

#[derive(Resource, Debug)]
pub struct LoadingAssets {
    pub handles: Vec<UntypedHandle>,
    pub max_confirmation_frames: u8,
    pub curr_confirmation_frames: u8,
}

#[derive(States, Default, Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub enum LevelLoadingState {
    #[default]
    LevelReady,
    LevelLoading,
}

impl LoadingAssets {
    pub fn new(max_frames: u8) -> Self {
        Self {
            handles: Vec::new(),
            max_confirmation_frames: max_frames,
            curr_confirmation_frames: 0,
        }
    }
}

pub fn check_loading_assets(
    mut load_state: ResMut<NextState<LevelLoadingState>>,
    mut loading: ResMut<LoadingAssets>,
    assets: Res<AssetServer>,
) {
    let _span = trace_span!("Checking Loading Status").entered();
    if !loading.handles.is_empty() {
        // Set confirmation frames to 0 just in case
        loading.curr_confirmation_frames = 0;

        loading.handles.retain(|handle| {
            // You'll thank me later for doing this recursively
            assets
                .get_recursive_dependency_load_state(handle)
                .is_none_or(|state| !state.is_loaded())
        });
    } else {
        loading.curr_confirmation_frames += 1;
        if loading.curr_confirmation_frames == loading.max_confirmation_frames {
            load_state.set(LevelLoadingState::LevelReady)
        }
    }
}

// In single player, we spawn just a single entity. Very simple
pub fn spawn_player_character(
    mut commands: Commands,
    game_kinds: Res<CurrentGameKind>,
    q_player: Single<(Entity, &PlayerInLobby)>,
) {
    let (player_ent, lobby_player) = (q_player.0, q_player.1);
    let mut rng = rand::rng();
    let pos = (rng.random_range(-50.0..50.0), rng.random_range(-50.0..50.0));
    let peer = PeerId::Local(0);

    let char = lobby_player.selected_character.unwrap();
    let player = Player {
        client: peer,
        character: char,
    };

    let player_character = spawn_game_object(
        &mut commands,
        game_kinds.0.unwrap(),
        Some(char),
        MultiPlayerComponentOptions::PREDICTED,
        (PlayerBaseBundle {
            player,
            position: Position(Vec2::new(pos.0, pos.1)),
            upgrade_slots: PlayerUpgradeSlots::new(5, 5),
            weapons: PlayerWeapons::default(),
            facing: CharacterFacing::default(),
        }),
    );
    add_weapon_to_character(
        player_character,
        char.starting_weapon(),
        &mut commands,
        game_kinds.0.unwrap(),
    );

    commands.entity(player_ent).remove::<PlayerInLobby>();
}

// In multiplayer, we spawn just a variety of entities based on their user attributes and the chosen player
pub fn spawn_characters_in_multiplayer(
    mut commands: Commands,
    game_kinds: Res<CurrentGameKind>,
    q_player: Query<(Entity, &PlayerInLobby, &RemoteId)>,
) {
    for (ent, lobby_player, peer) in q_player {
        let mut rng = rand::rng();
        let pos = (rng.random_range(-50.0..50.0), rng.random_range(-50.0..50.0));

        let char = lobby_player.selected_character.unwrap();
        let player = Player {
            client: peer.0,
            character: char,
        };

        let player = spawn_game_object(
            &mut commands,
            game_kinds.0.unwrap(),
            Some(char),
            MultiPlayerComponentOptions::PREDICTED,
            (
                PlayerBaseBundle {
                    player,
                    position: Position(Vec2::new(pos.0, pos.1)),
                    upgrade_slots: PlayerUpgradeSlots::new(5, 5),
                    weapons: PlayerWeapons::default(),
                    facing: CharacterFacing::default(),
                },
                ControlledBy {
                    owner: ent,
                    lifetime: Lifetime::default(),
                },
            ),
        );
        add_weapon_to_character(
            player,
            char.starting_weapon(),
            &mut commands,
            game_kinds.0.unwrap(),
        );
    }
}
