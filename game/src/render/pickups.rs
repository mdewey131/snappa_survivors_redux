use avian2d::prelude::Position;
use bevy::{asset::UntypedAssetId, prelude::*};

use crate::{
    render::RenderYtoZ,
    shared::{
        loading::{LevelLoadingState, track_loading_asset},
        pickups::{HealthPickup, XPPickup},
    },
};

pub struct SharedPickupsRenderPlugin;

impl Plugin for SharedPickupsRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(LevelLoadingState::LevelLoading),
            load_pickup_render_assets.pipe(track_loading_asset),
        )
        .add_observer(xp_pickup_spawned)
        .add_observer(hp_pickup_spawned);
    }
}

#[derive(Resource)]
pub struct PickupRenderAssets {
    xp: Handle<Image>,
    hp: Handle<Image>,
}

fn load_pickup_render_assets(
    mut commands: Commands,
    assets: Res<AssetServer>,
) -> Vec<UntypedHandle> {
    let xp: Handle<Image> = assets.load("pickups/xp_orb.png");
    let xp_ret = xp.clone();

    let hp: Handle<Image> = assets.load("pickups/hp_pickup.png");
    let hp_ret = hp.clone();
    commands.insert_resource(PickupRenderAssets { xp, hp });
    vec![xp_ret.into(), hp_ret.into()]
}

fn xp_pickup_spawned(
    trigger: On<Add, XPPickup>,
    mut commands: Commands,
    pickups: Res<PickupRenderAssets>,
    q_pickup: Query<&Position, With<XPPickup>>,
) {
    if let Ok(pos) = q_pickup.get(trigger.entity) {
        commands.entity(trigger.entity).insert((
            Sprite::from(pickups.xp.clone()),
            Transform::from_translation(pos.extend(pos.y)),
            RenderYtoZ::default(),
        ));
    }
}

fn hp_pickup_spawned(
    trigger: On<Add, HealthPickup>,
    mut commands: Commands,
    pickups: Res<PickupRenderAssets>,
    q_pickup: Query<&Position, With<HealthPickup>>,
) {
    if let Ok(pos) = q_pickup.get(trigger.entity) {
        commands.entity(trigger.entity).insert((
            Sprite::from(pickups.hp.clone()),
            Transform::from_translation(pos.extend(pos.y)),
            RenderYtoZ::default(),
        ));
    }
}
