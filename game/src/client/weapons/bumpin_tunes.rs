use bevy::prelude::*;

use crate::shared::{
    game_kinds::DefaultClientFilter, weapons::bumpin_tunes::bumpin_tunes_activate,
};

pub struct ClientBumpinTunesPlugin;

impl Plugin for ClientBumpinTunesPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(bumpin_tunes_activate::<DefaultClientFilter>);
    }
}
