use crate::shared::states::AppState;
use bevy::prelude::*;

const EXPECTED_LOADING_CONFIRMATION_FRAMES: u8 = 30;

/// The set of common functions that are needed by both server and client.
///
/// This is primarily responsible to keep track of what's available in `LoadingAssets`
pub struct SharedLoadingPlugin;
impl Plugin for SharedLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LevelLoadingState>()
            .insert_resource(LoadingAssets::new(EXPECTED_LOADING_CONFIRMATION_FRAMES))
            .add_systems(OnEnter(AppState::LoadingLevel), start_level_loading_state)
            .add_systems(
                Update,
                (check_loading_assets,).run_if(
                    in_state(AppState::LoadingLevel).and(in_state(LevelLoadingState::LevelLoading)),
                ),
            );
    }
}

#[derive(Resource, Debug)]
pub struct LoadingAssets {
    pub handles: Vec<UntypedHandle>,
    pub max_confirmation_frames: u8,
    pub curr_confirmation_frames: u8,
}

/// A function to be used whenever we want to track an asset.
///
/// I expect other systems to return the untyped asset handles they're loading
/// for the sake of tracking.
///
/// Architecting it this way allows for common loading tracking paradigm
/// across server and client
pub fn track_loading_asset(
    mut assets_in: In<Vec<UntypedHandle>>,
    mut loading: ResMut<LoadingAssets>,
) {
    loading.handles.append(&mut assets_in);
}

#[derive(States, Default, Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub enum LevelLoadingState {
    #[default]
    NotLoading,
    LevelReady,
    LevelLoading,
}
fn start_level_loading_state(mut state: ResMut<NextState<LevelLoadingState>>) {
    state.set(LevelLoadingState::LevelLoading);
}

impl LoadingAssets {
    pub fn new(max_frames: u8) -> Self {
        Self {
            handles: Vec::new(),
            max_confirmation_frames: max_frames,
            curr_confirmation_frames: 0,
        }
    }
}

pub fn check_loading_assets(
    mut load_state: ResMut<NextState<LevelLoadingState>>,
    mut loading: ResMut<LoadingAssets>,
    assets: Res<AssetServer>,
) {
    let _span = info_span!("Checking Loading Status").entered();
    if !loading.handles.is_empty() {
        // Set confirmation frames to 0 just in case
        loading.curr_confirmation_frames = 0;

        loading.handles.retain(|handle| {
            // You'll thank me later for doing this recursively
            assets
                .get_recursive_dependency_load_state(handle)
                .is_none_or(|state| !state.is_loaded())
        });
    } else {
        loading.curr_confirmation_frames += 1;
        if loading.curr_confirmation_frames == loading.max_confirmation_frames {
            load_state.set(LevelLoadingState::LevelReady)
        }
    }
}
