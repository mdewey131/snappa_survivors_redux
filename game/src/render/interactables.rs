use bevy::prelude::*;

use crate::shared::interactables::BeerShrine;

pub struct BeerShrineRenderPlugin;

impl Plugin for BeerShrineRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_beer_shrine);
    }
}

fn animate_beer_shrine(mut commands: Commands, mut q_shrine: Query<(&mut Sprite, &BeerShrine)>) {
    for (mut sprite, shrine) in &mut q_shrine {
        let pct_charge_remaining = shrine.current_charge / shrine.max_charge;
        // TODO: Change
        let frames = 7.0;
        let atlas = if let Some(ref mut a) = sprite.texture_atlas {
            a
        } else {
            panic!("Attempted to animate a beer shrine without a sprite");
        };
        atlas.index = (frames - pct_charge_remaining * frames).round() as usize;
        trace!("Set index to {}", atlas.index);
    }
}
