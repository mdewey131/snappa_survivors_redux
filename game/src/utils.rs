use std::f32::consts::TAU;

#[cfg(feature = "dev")]
pub mod zoo;

use bevy::{
    ecs::{
        entity::MapEntities,
        system::{SystemId, SystemInput},
    },
    prelude::*,
};

use rand::Rng;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
/// Reads the path that is provided and imports RON, returning
/// a concrete instance of type T
pub fn read_ron<T: DeserializeOwned>(path: String) -> T {
    if let Ok(s) = std::fs::read_to_string(&path) {
        ron::from_str::<T>(&s).expect("Failed to Deserialize Type")
    } else {
        panic!("Failed to read file {:?}", &path);
    }
}

/// A small component that marks something that has a callback with some input
#[derive(Component, Clone, Copy)]
pub struct CallbackWithInput<I: SystemInput>(pub SystemId<I, ()>);

/// Small convenient struct to use for From<> derivations and in order to import assets at file destinations
/// that get joined with this
#[derive(Deref)]
pub struct AssetFolder(pub String);
impl AssetFolder {
    pub fn to_path(&self, path: String) -> String {
        format!("{}/{}", self.0, path)
    }
}
impl From<()> for AssetFolder {
    fn from(_value: ()) -> Self {
        AssetFolder("".into())
    }
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Reflect)]
#[relationship(relationship_target = CreatorOf)]
pub struct CreatedBy(pub Entity);

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = CreatedBy)]
pub struct CreatorOf(Vec<Entity>);

impl MapEntities for CreatedBy {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        self.0 = entity_mapper.get_mapped(self.0);
    }
}

/// Describes how to spawn a group of things, returning the positions at which to spawn them.
/// Because this is a utility for spawning groups, there is no single option
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Default)]
pub enum SpawnPattern {
    Circle {
        amount: u8,
        center: Vec2,
        radius: f32,
        radius_only: bool,
    },
}

impl Default for SpawnPattern {
    fn default() -> Self {
        Self::Circle {
            amount: 1,
            center: Vec2::ZERO,
            radius: 0.0,
            radius_only: true,
        }
    }
}

impl SpawnPattern {
    pub fn to_positions(&self) -> Vec<Vec2> {
        match self {
            Self::Circle {
                amount,
                center,
                radius,
                radius_only,
            } => {
                let mut rng = rand::rng();
                let mut to_ret = Vec::new();

                for _ in 0..(*amount) {
                    let angle = rng.random_range(-TAU..TAU);
                    let length = if *radius_only {
                        *radius
                    } else {
                        rng.random_range(0.0..*radius)
                    };
                    let new_vec = (Vec2::from_angle(angle) * length) + center;
                    to_ret.push(new_vec)
                }
                to_ret
            }
        }
    }
    /// In the event that you have a custom centerpoint to use, this
    /// will return the relative position of spawning elements for you, hiding the ugliness.
    pub fn positions_from_centerpoint(&self, _centerpoint: Vec2) -> Vec<Vec2> {
        // Kind of hacky maybe, but simplifies the code a lot
        let pattern_override = match self {
            Self::Circle {
                amount,
                center: _,
                radius,
                radius_only,
            } => Self::Circle {
                amount: *amount,
                center: Vec2::ZERO,
                radius: *radius,
                radius_only: *radius_only,
            },
            _ => *self,
        };
        pattern_override.to_positions()
    }
}
