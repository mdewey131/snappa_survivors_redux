use avian2d::prelude::*;
use bevy::prelude::*;

use crate::shared::colliders::{ColliderTypes, CommonColliderBundle};

#[derive(Component)]
pub struct XPDrop(pub f32);

pub struct SharedDropsPlugin;
impl Plugin for SharedDropsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(add_xp_collider_components);
    }
}

pub fn add_xp_collider_components(trig: On<Add, XPDrop>, mut commands: Commands) {
    commands.entity(trig.entity).insert((
        CommonColliderBundle::new(
            RigidBody::Kinematic,
            Collider::circle(10.0),
            1.0,
            [ColliderTypes::RemotePickup].into(),
            [ColliderTypes::Player, ColliderTypes::PlayerPickupRadius].into(),
        ),
        Sensor,
    ));
}
