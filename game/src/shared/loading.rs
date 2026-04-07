use crate::shared::{
    combat::CharacterFacing, game_kinds::*, game_object_spawning::*, lobby::PlayerInLobby,
    players::*, upgrades::PlayerUpgradeSlots, weapons::*,
};
use avian2d::prelude::*;
use bevy::{asset::LoadState, prelude::*};
use lightyear::prelude::*;
use rand::Rng;

#[derive(Resource, Debug)]
pub struct LoadingAssets {
    pub handles: Vec<UntypedHandle>,
}

pub fn check_loading_assets(mut loading: ResMut<LoadingAssets>, assets: Res<AssetServer>) {
    let _span = trace_span!("Checking Loading Status").entered();
    let mut to_rm = Vec::new();
    for (i, handle) in loading.handles.iter().enumerate() {
        let load_state = assets
            .get_load_state(handle)
            .expect("Looking for a handle that doesn't exist!");

        match load_state {
            LoadState::Loaded => to_rm.push(i),
            LoadState::Loading => {}
            LoadState::NotLoaded => {}
            LoadState::Failed(_error) => {}
        }
    }

    for i in to_rm.iter().rev() {
        loading.handles.remove(*i);
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
