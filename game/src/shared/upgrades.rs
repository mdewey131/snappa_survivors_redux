use std::fmt::Display;

use crate::shared::{
    GameMainChannel,
    game_kinds::{CurrentGameKind, SinglePlayer, is_single_player},
    players::{CharacterKind, Player, PlayerWeapons},
    states::{AppState, InGameState},
    stats::{RawStatsList, StatKind, StatList, xp::LevelUpMessage},
    weapons::{Weapon, WeaponKind, add_weapon_to_player},
};
use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use lightyear::prelude::*;
use rand::{
    Rng,
    distr::{Distribution, StandardUniform},
};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[cfg(feature = "dev")]
mod editor;
mod upgrade_manager;

#[cfg(feature = "dev")]
pub use editor::*;
pub use upgrade_manager::*;

/// TO BE MOVED to its proper folder
pub struct ClientUpgradePlugin;
impl Plugin for ClientUpgradePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnExit(InGameState::SelectingUpgrades),
            apply_upgrade.run_if(is_single_player),
        )
        .add_systems(
            OnEnter(AppState::InGame),
            add_upgrade_manager.run_if(is_single_player),
        )
        .add_systems(
            Update,
            (
                (
                    client_move_to_selecting_upgrades_state_on_server_message,
                    client_send_upgrade_selection_message,
                    client_move_to_in_game_state_on_receive_server_start_game_message,
                )
                    .run_if(not(is_single_player)),
                (
                    spawn_upgrade_choices_on_level_up
                        .pipe(client_move_to_selecting_upgrades_state_on_upgrade_generation),
                    client_1p_move_to_in_game_state_on_upgrade_selection,
                )
                    .run_if(is_single_player),
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

pub struct DedicatedServerUpgradePlugin;
impl Plugin for DedicatedServerUpgradePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            add_upgrade_manager.run_if(not(is_single_player)),
        )
        .add_systems(
            OnExit(InGameState::SelectingUpgrades),
            apply_upgrade.run_if(not(is_single_player)),
        );
        app.add_systems(
            Update,
            (
                spawn_upgrade_choices_on_level_up
                    .pipe(server_send_upgrade_message_to_client)
                    .run_if(in_state(InGameState::InGame)),
                (
                    server_on_receive_upgrade_selection_message,
                    server_send_start_game_message_on_all_selected.run_if(all_players_selected),
                )
                    .run_if(in_state(InGameState::SelectingUpgrades)),
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

pub struct TempUpgradePlugin;
impl Plugin for TempUpgradePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<UpgradeSelectionMessage>();
        app.register_message::<ServerMoveToUpgradesMessage>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<UpgradeSelectionMessage>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<ServerStartGameMessage>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_component::<UpgradeOptions>().add_prediction();
        #[cfg(feature = "dev")]
        app.add_plugins(UpgradeEditorPlugin);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerMoveToUpgradesMessage;

#[derive(Message, Clone, Debug, Serialize, Deserialize)]
pub struct UpgradeSelectionMessage(pub usize);

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq)]
pub struct ServerStartGameMessage;

/// This component is added on to player entities.
/// We do it in this way so that we can know which ones to render on each player's screen,
/// and becuase it makes reasoning about which ones are controlled vs. not unnecessary,
/// which is what we want since this can exist in SP or MP
#[derive(Component, Reflect, Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct UpgradeOptions {
    pub options: [Upgrade; 3],
    pub selected: Option<usize>,
}

/// The component that marks a given upgrade.
/// Players will be offered one of three choices for them
/// to take, which will boost their stats depending on
/// the upgrade kind (which provides base values),
/// and the rarity (which modifies those values)
#[derive(Reflect, Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Upgrade {
    pub kind: UpgradeKind,
    pub rarity: UpgradeRarity,
    pub level: u8,
    pub rewards: Vec<UpgradeReward>,
}

#[derive(Reflect, Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub enum UpgradeRarity {
    #[default]
    Common,
    Rare,
    Epic,
    Legendary,
}
impl Distribution<UpgradeRarity> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> UpgradeRarity {
        match rng.random_range((0..4)) {
            0 => UpgradeRarity::Common,
            1 => UpgradeRarity::Rare,
            2 => UpgradeRarity::Epic,
            _ => UpgradeRarity::Legendary,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Hash, Eq)]
#[reflect(Default)]
pub enum UpgradeKind {
    AddWeapon(WeaponKind),
    UpgradeWeapon(WeaponKind),
    UpgradePlayerStat(StatUpgradeKind),
}
impl Default for UpgradeKind {
    fn default() -> Self {
        Self::UpgradePlayerStat(StatUpgradeKind::default())
    }
}

#[derive(
    Reflect, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default, Hash, Eq, EnumIter,
)]
#[reflect(Default)]
pub enum StatUpgradeKind {
    #[default]
    Armor,
    CritChance,
    CDR,
    Damage,
    EffDuration,
    EffSize,
    Evasion,
    MaxHealth,
    HealthRegen,
    Luck,
    MoveSpeed,
    PickupRadius,
    ProjectileCount,
    Shield,
    Thorns,
    XPGain,
}
impl From<StatUpgradeKind> for StatKind {
    fn from(suk: StatUpgradeKind) -> Self {
        match suk {
            StatUpgradeKind::Armor => StatKind::Armor,
            StatUpgradeKind::CritChance => StatKind::CritChance,
            StatUpgradeKind::CDR => StatKind::CDR,
            StatUpgradeKind::Damage => StatKind::Damage,
            StatUpgradeKind::EffDuration => StatKind::EffDuration,
            StatUpgradeKind::EffSize => StatKind::EffSize,
            StatUpgradeKind::Evasion => StatKind::Evasion,
            StatUpgradeKind::MaxHealth => StatKind::Health,
            StatUpgradeKind::HealthRegen => StatKind::HealthRegen,
            StatUpgradeKind::Luck => StatKind::Luck,
            StatUpgradeKind::MoveSpeed => StatKind::MS,
            StatUpgradeKind::PickupRadius => StatKind::PickupR,
            StatUpgradeKind::ProjectileCount => StatKind::ProjCount,
            StatUpgradeKind::Shield => StatKind::Shield,
            StatUpgradeKind::Thorns => StatKind::Thorns,
            StatUpgradeKind::XPGain => StatKind::XPGain,
        }
    }
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct StatsUpgrades(Vec<(StatKind, f32)>);

#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
pub struct PlayerUpgradeSlots {
    pub weapons: HashMap<WeaponKind, u8>,
    pub weapon_limit: usize,
    pub stats: HashMap<StatUpgradeKind, u8>,
    pub stats_limit: usize,
}
impl PlayerUpgradeSlots {
    pub fn new(weapon_limit: usize, stats_limit: usize) -> Self {
        Self {
            weapons: HashMap::new(),
            weapon_limit,
            stats: HashMap::new(),
            stats_limit,
        }
    }
}

/// The specific portion of the upgrades process that reads level up messages and spawns choices
///
/// This is to be run on the server and received by the client when we're in multiplayer,
/// but its meant to be generated by the client when we're in single player mode
///
/// This returns a result, which we mostly do to be able to pipe this into other functions that
/// do different things depending on where we are (send message from server in MP, move to selecting state on client)
pub fn spawn_upgrade_choices_on_level_up(
    mut reader: MessageReader<LevelUpMessage>,
    mut commands: Commands,
    mut manager: ResMut<UpgradeManager>,
    q_player: Query<(Entity, &PlayerUpgradeSlots), With<Player>>,
) -> Result<(), BevyError> {
    if let Some(m) = reader.read().next() {
        for (p_ent, c_upgrades) in &q_player {
            let comp_options = manager.generate_upgrade_options(c_upgrades);
            commands.entity(p_ent).insert(comp_options);
        }
        Ok(())
    } else {
        Err(BevyError::from("no_op"))
    }
}

/// Run on the server. We expect the values of the selection upgrades to be piped
/// in because we need to attach networking components
pub fn server_send_upgrade_message_to_client(
    incoming: In<Result<(), BevyError>>,
    mut next: ResMut<NextState<InGameState>>,
    mut q_messages: Single<&mut MessageSender<ServerMoveToUpgradesMessage>>,
) {
    // Guard clause
    if incoming.0.is_err() {
        return;
    }
    info!("Sending message to client");
    q_messages.send::<GameMainChannel>(ServerMoveToUpgradesMessage);
    next.set(InGameState::SelectingUpgrades)
}

pub fn client_move_to_selecting_upgrades_state_on_upgrade_generation(
    incoming: In<Result<(), BevyError>>,
    mut next: ResMut<NextState<InGameState>>,
) {
    if incoming.0.is_err() {
        return;
    }
    next.set(InGameState::SelectingUpgrades)
}

pub fn client_move_to_selecting_upgrades_state_on_server_message(
    mut next: ResMut<NextState<InGameState>>,
    mut q_rec: Single<&mut MessageReceiver<ServerMoveToUpgradesMessage>>,
) {
    for mut _m in q_rec.receive() {
        next.set(InGameState::SelectingUpgrades);
        break;
    }
}

pub fn server_on_receive_upgrade_selection_message(
    mut q_server: Query<&mut MessageReceiver<UpgradeSelectionMessage>>,
    mut q_players: Query<(&ControlledBy, &mut UpgradeOptions)>,
) {
    for (cont, mut options) in &mut q_players {
        if let Ok((mut messages)) = q_server.get_mut(cont.owner) {
            if let Some(m) = messages.receive().next() {
                options.selected = Some(m.0)
            }
        }
    }
}

pub fn client_send_upgrade_selection_message(
    mut upgrade_messages: MessageReader<UpgradeSelectionMessage>,
    mut q_sender: Single<&mut MessageSender<UpgradeSelectionMessage>>,
) {
    if let Some(message) = upgrade_messages.read().next() {
        q_sender.send::<GameMainChannel>(message.clone());
    }
}

pub fn client_1p_move_to_in_game_state_on_upgrade_selection(
    mut upgrade_messages: MessageReader<UpgradeSelectionMessage>,
    mut state: ResMut<NextState<InGameState>>,
    mut q_player: Single<&mut UpgradeOptions, With<Player>>,
) {
    if let Some(message) = upgrade_messages.read().next() {
        q_player.selected = Some(message.0);
        state.set(InGameState::InGame)
    }
}

pub fn client_move_to_in_game_state_on_receive_server_start_game_message(
    mut state: ResMut<NextState<InGameState>>,
    mut q_message: Single<&mut MessageReceiver<ServerStartGameMessage>>,
) {
    if q_message.receive().next().is_some() {
        state.set(InGameState::InGame)
    }
}

pub fn server_send_start_game_message_on_all_selected(
    mut state: ResMut<NextState<InGameState>>,
    mut q_sender: Single<&mut MessageSender<ServerStartGameMessage>>,
) {
    q_sender.send::<GameMainChannel>(ServerStartGameMessage);
    state.set(InGameState::InGame)
}

fn all_players_selected(q_players: Query<&UpgradeOptions>) -> bool {
    q_players.iter().all(|comp| comp.selected.is_some())
}

pub fn apply_upgrade(
    mut commands: Commands,
    game_kind: Res<CurrentGameKind>,
    mut q_upgrade_options: Query<(
        Entity,
        &mut UpgradeOptions,
        &mut PlayerUpgradeSlots,
        &mut StatList,
        &PlayerWeapons,
    )>,
    mut q_weapon_stats: Query<(&mut StatList), Without<UpgradeOptions>>,
) {
    for (ent, mut options, mut slots, mut player_stats, weapons) in &mut q_upgrade_options {
        let index = options.selected.unwrap();
        let m_selected = options.options.get_mut(index);
        let selected = m_selected.unwrap();
        let mut stats_list = match selected.kind {
            UpgradeKind::UpgradeWeapon(w) => q_weapon_stats
                .get_mut(*(weapons.0.get(&w).unwrap()))
                .unwrap(),
            _ => player_stats,
        };
        let rewards = selected.rewards.drain(..);
        for reward in rewards {
            match reward {
                UpgradeReward::AddWeapon(w) => {
                    add_weapon_to_player(ent, w, &mut commands, game_kind.0.unwrap());
                }
                UpgradeReward::StatUpgrade { range, kind, value } => {
                    let sk = StatKind::from(kind);
                    let mut stat = stats_list
                        .list
                        .get_mut(&sk)
                        .expect(&format!("This entity is expected to have {:?}", sk));
                    stat.base_value += value.unwrap();
                }
                _ => todo!(),
            }
        }

        match selected.kind {
            UpgradeKind::AddWeapon(w) => slots.weapons.insert(w, selected.level),
            UpgradeKind::UpgradeWeapon(w) => slots.weapons.insert(w, selected.level),
            UpgradeKind::UpgradePlayerStat(s) => slots.stats.insert(s, selected.level),
        };
    }
}
