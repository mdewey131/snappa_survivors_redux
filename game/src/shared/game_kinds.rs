use bevy::prelude::*;
use lightyear::prelude::*;

#[derive(Debug, Clone, Copy, Reflect)]
pub enum GameKinds {
    SinglePlayer,
    MultiPlayer,
}

pub type DefaultClientFilter = Or<(With<Predicted>, With<SinglePlayer>)>;
pub type DefaultServerFilter = With<Replicate>;

/// The marker component and types that is used to differentiate between
/// We will have lightyear do the work of making predicted and replicated
#[derive(Component, Debug, Clone, Copy)]
pub struct SinglePlayer;

#[derive(Resource, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Resource)]
pub struct CurrentGameKind(pub Option<GameKinds>);

pub struct GameKindsPlugin;

impl Plugin for GameKindsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentGameKind>();
    }
}

pub fn is_single_player(r: Res<CurrentGameKind>) -> bool {
    if let Some(k) = r.0 {
        match k {
            GameKinds::SinglePlayer => true,
            _ => false,
        }
    } else {
        false
    }
}

/// Used to add the lightyear relevant components that are needed on this entity.
/// The general expectation is that you derive From<C> on this and pass it into
/// the `add_game_kinds_components` function
pub struct MultiPlayerComponentOptions {
    pub pred: bool,
    pub interp: bool,
}
/// Inserts the compoonents that this entity needs related to the game kind
pub fn add_game_kinds_components(
    comms: &mut Commands,
    ent: Entity,
    game_kind: GameKinds,
    m_options: MultiPlayerComponentOptions,
) {
    match game_kind {
        GameKinds::SinglePlayer => {
            comms.entity(ent).insert(SinglePlayer);
        }
        GameKinds::MultiPlayer => {
            comms
                .entity(ent)
                .insert(Replicate::to_clients(NetworkTarget::All));
            if m_options.pred {
                comms
                    .entity(ent)
                    .insert(PredictionTarget::to_clients(NetworkTarget::All));
            }
            if m_options.interp {
                comms
                    .entity(ent)
                    .insert(InterpolationTarget::to_clients(NetworkTarget::All));
            }
        }
    }
}
