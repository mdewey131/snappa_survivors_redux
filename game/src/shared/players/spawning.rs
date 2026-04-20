use super::*;
use crate::shared::{
    game_kinds::CurrentGameKind, game_object_spawning::*, lobby::PlayerInLobby,
    weapons::add_weapon_to_character,
};

use bevy::prelude::*;
use lightyear::prelude::*;
use rand::Rng;

// In multiplayer, we spawn just a variety of entities based on their user attributes and the chosen player.
// This expects an input list of possible positions to take
pub fn spawn_characters(
    mut pos_in: &mut Vec<Vec2>,
    mut commands: &mut Commands,
    game_kinds: &Res<CurrentGameKind>,
    q_player: &Query<(Entity, &PlayerInLobby, Option<&RemoteId>)>,
) {
    for (ent, lobby_player, m_peer) in q_player {
        let pos = pos_in.pop().expect("No position found for player!");

        let client = m_peer.map(|p| p.0);
        let char = lobby_player.selected_character.unwrap();
        let player = Player {
            client: client,
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
                    position: Position(Vec2::new(pos.x, pos.y)),
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
        // This line replicates the logic that I had back when single player and multiplayer character spawning
        // were separated. This may not be necessary, or it may be applicable in either scenario!
        if m_peer.is_none() {
            commands.entity(ent).remove::<PlayerInLobby>();
        }
    }
}
