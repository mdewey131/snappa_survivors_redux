use bevy::prelude::*;

use crate::{
    render::weapons::*,
    shared::{
        combat::CombatSystemSet,
        game_kinds::{DefaultClientFilter, is_single_player},
        states::InGameState,
        weapons::throw_hands::*,
    },
};

pub struct ClientThrowHandsPlugin;
impl Plugin for ClientThrowHandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientThrowHandsRenderPlugin)
            .add_systems(
                FixedUpdate,
                update_attack::<DefaultClientFilter>
                    .run_if(in_state(InGameState::InGame).and(is_single_player))
                    .in_set(CombatSystemSet::Combat),
            )
            .add_observer(on_activate::<DefaultClientFilter>)
            .add_observer(on_deactivate::<DefaultClientFilter>);
    }
}

pub struct ClientThrowHandsRenderPlugin;
impl Plugin for ClientThrowHandsRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(add_throwing_hands_attack_sprite::<DefaultClientFilter>)
            .add_systems(Update, throwing_hands_sprite_follow::<DefaultClientFilter>);
    }
}
