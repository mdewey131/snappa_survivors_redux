//! Handles the integration with bei
use crate::shared::Player;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use lightyear::prelude::input::{InputConfig, InputRegistryExt, bei::InputPlugin};

pub struct GameInputProtocolPlugin;

impl Plugin for GameInputProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputPlugin::<Player> {
            config: InputConfig::<Player> {
                rebroadcast_inputs: true,
                ..default()
            },
        });

        app.register_input_action::<Movement>();
    }
}

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub struct Movement;
