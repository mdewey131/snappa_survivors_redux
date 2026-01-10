use bevy::{
    ecs::system::{SystemId, SystemInput},
    prelude::*,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
/// Reads the path that is provided and imports RON, returning
/// a concrete instance of type T
pub fn read_ron<T: DeserializeOwned>(path: String) -> T {
    if let Ok(s) = std::fs::read_to_string(&path) {
        let val = ron::from_str::<T>(&s).expect("Failed to Deserialize Type");
        val
    } else {
        panic!("Failed to read file {:?}", &path);
    }
}

/// A small component that marks something that has a callback with some input
#[derive(Component, Clone, Copy)]
pub struct CallbackWithInput<I: SystemInput>(pub SystemId<I, ()>);
