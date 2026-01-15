use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use lightyear::prelude::{Controlled, Predicted, Replicate};

use crate::{
    render::player::rendering_on_player_add,
    shared::{
        colliders::CommonColliderBundle,
        combat::CombatSystemSet,
        game_kinds::SinglePlayer,
        inputs::Movement,
        players::{Player, player_movement},
    },
};

pub struct ClientPlayerPlugin;

impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (player_movement::<Or<(With<Predicted>, With<SinglePlayer>)>>)
                .in_set(CombatSystemSet::Combat),
        )
        .add_observer(handle_predicted_player_spawn)
        .add_observer(handle_sp_player_spawn);
    }
}

pub struct ClientPlayerRenderPlugin;
impl Plugin for ClientPlayerRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            rendering_on_player_add::<Or<(With<SinglePlayer>, With<Predicted>)>>,
        );
    }
}

fn handle_predicted_player_spawn(
    trigger: On<Add, Player>,
    mut commands: Commands,
    q_pred: Query<(Has<Controlled>, &Player), With<Predicted>>,
) {
    if let Ok((cont, p)) = q_pred.get(trigger.entity) {
        if cont {
            commands.spawn((
                ActionOf::<Player>::new(trigger.entity),
                Action::<Movement>::new(),
                Bindings::spawn(Cardinal::wasd_keys()),
                // This isn't in the example, but
                // it seems that you need this so that the
                // replication works in a single player scenario. It doesn't appear
                // to affect MP too much
                Replicate::to_server(),
            ));
        }
        // regardless, add the collider components
        commands
            .entity(trigger.entity)
            .insert(CommonColliderBundle::from(*p));
    }
}

fn handle_sp_player_spawn(
    trigger: On<Add, Player>,
    mut commands: Commands,
    q_pred: Query<&Player, With<SinglePlayer>>,
) {
    if let Ok(p) = q_pred.get(trigger.entity) {
        commands.spawn((
            ActionOf::<Player>::new(trigger.entity),
            Action::<Movement>::new(),
            Bindings::spawn(Cardinal::wasd_keys()),
            // This isn't in the example, but
            // it seems that you need this so that the
            // replication works in a single player scenario. It doesn't appear
            // to affect MP too much
            Replicate::to_server(),
        ));
        commands
            .entity(trigger.entity)
            .insert(CommonColliderBundle::from(*p));
    }
}
