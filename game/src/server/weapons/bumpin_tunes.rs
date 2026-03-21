use bevy::prelude::*;

use crate::shared::{
    game_kinds::DefaultServerFilter, weapons::bumpin_tunes::bumpin_tunes_activate,
};

pub struct DedicatedServerBumpinTunesPlugin;

impl Plugin for DedicatedServerBumpinTunesPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(bumpin_tunes_activate::<DefaultServerFilter>);
    }
}
