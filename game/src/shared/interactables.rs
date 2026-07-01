use avian2d::prelude::{Collider, Collisions, LayerMask, RigidBody};
use bevy::{ecs::relationship::Relationship, prelude::*};

use crate::shared::{
    colliders::{ColliderTypes, CommonColliderBundle},
    players::Player,
    states::{AppState, InGameState},
    upgrades::UpgradeManager,
};

pub struct SharedInteractablesPlugin;

impl Plugin for SharedInteractablesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            charge_beer_shrine
                .run_if(resource_exists::<UpgradeManager>.and(in_state(InGameState::InGame))),
        );
    }
}

/// Marker struct for an element in the map that can be interacted with.
/// This unifies the underlying different kinds, similar to the way that
/// `Weapon` works
#[derive(Component, Default)]
pub struct Interactable;

#[derive(Component)]
#[require(Interactable, DespawnOnEnter<AppState> = DespawnOnEnter(AppState::GameOver))]
pub struct BeerShrine {
    pub max_charge: f32,
    pub current_charge: f32,
    pub charge_rate_secs: f32,
}
pub fn beer_shrine_collider() -> CommonColliderBundle {
    CommonColliderBundle::new(
        RigidBody::Static,
        Collider::rectangle(48.0, 96.0),
        1000.0,
        [ColliderTypes::SolidObject].into(),
        LayerMask::ALL,
    )
}

#[derive(Component, Debug)]
pub struct BeerShrineChargeRadius;
pub fn beer_shrine_collider_detection_range() -> CommonColliderBundle {
    CommonColliderBundle::new(
        RigidBody::Static,
        Collider::circle(500.0),
        0.0,
        [ColliderTypes::StaticPickup].into(),
        [ColliderTypes::Player].into(),
    )
}

pub fn charge_beer_shrine(
    mut commands: Commands,
    mut manager: ResMut<UpgradeManager>,
    cols: Collisions,
    game_time: Res<Time<Virtual>>,
    q_charge_radius: Query<(Entity, &ChildOf), With<BeerShrineChargeRadius>>,
    q_players: Query<(Entity), (Without<BeerShrineChargeRadius>, With<Player>)>,
    mut q_shrine: Query<&mut BeerShrine>,
) {
    for (radius, parent) in q_charge_radius {
        let mut n_players = 0;
        for collision in cols.collisions_with(radius) {
            if q_players.get(collision.collider1).is_ok()
                || q_players.get(collision.collider2).is_ok()
            {
                n_players += 1;
            }
        }
        let mut shrine = q_shrine.get_mut(parent.get()).expect("Shrine not found!");
        if n_players >= 1 {
            let charge_to_sub =
                (shrine.charge_rate_secs * n_players as f32) * game_time.delta_secs();
            shrine.current_charge -= charge_to_sub;
            if shrine.current_charge < 0.0 {
                info!("Shrine Charged - Despawning Outer Collider!");
                commands.entity(radius).despawn();
                let players = q_players.iter().collect();
                manager.add_shrine_rewards_to_queue(players);
            }
        } else {
            shrine.current_charge = shrine.max_charge
        }
    }
}
