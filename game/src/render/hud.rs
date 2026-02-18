use std::marker::PhantomData;

use crate::shared::{
    game_kinds::{DefaultClientFilter, SinglePlayer},
    players::Player,
    states::*,
    stats::{components::*, xp::LevelManager, *},
    upgrades::{PlayerUpgradeSlots, StatUpgradeKind},
    weapons::WeaponKind,
};
use bevy::prelude::*;
use lightyear::prelude::{Controlled, Predicted};

mod stat_display_trait;
use stat_display_trait::*;

pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        // Add plugins related to display stats
        app.add_plugins((
            StatDisplayPlugin::<AttackRange>::new(),
            StatDisplayPlugin::<Armor>::new(),
            StatDisplayPlugin::<CritChance>::new(),
            StatDisplayPlugin::<CritDamage>::new(),
            StatDisplayPlugin::<CooldownRate>::new(),
            StatDisplayPlugin::<Damage>::new(),
            StatDisplayPlugin::<EffectDuration>::new(),
            StatDisplayPlugin::<EffectSize>::new(),
            StatDisplayPlugin::<Evasion>::new(),
            StatDisplayPlugin::<HealthRegen>::new(),
            StatDisplayPlugin::<Luck>::new(),
            StatDisplayPlugin::<LifeSteal>::new(),
            StatDisplayPlugin::<MovementSpeed>::new(),
            StatDisplayPlugin::<PickupRadius>::new(),
            StatDisplayPlugin::<ProjectileCount>::new(),
        ));

        app.add_plugins((
            StatDisplayPlugin::<ProjectileSpeed>::new(),
            StatDisplayPlugin::<Shield>::new(),
            StatDisplayPlugin::<Thorns>::new(),
            StatDisplayPlugin::<XPGain>::new(),
        ));

        app.add_systems(OnEnter(AppState::InGame), spawn_hud_container)
            .add_systems(
                Update,
                (
                    update_health_bar,
                    update_xp_bar,
                    update_game_clock_display,
                    update_upgrade_slot_display,
                )
                    .run_if(in_state(InGameState::InGame)),
            )
            .add_systems(OnExit(InGameState::InGame), toggle_hud)
            .add_systems(
                OnEnter(InGameState::InGame),
                toggle_hud.run_if(hud_not_shown),
            );
    }
}

#[derive(Component, Debug)]
#[require(Node = outer_hud_node())]
pub struct OuterHudContainer;
fn outer_hud_node() -> Node {
    Node {
        display: Display::Grid,
        grid_template_columns: vec![RepeatedGridTrack::percent(20, 5.0)],
        grid_template_rows: vec![RepeatedGridTrack::percent(20, 5.0)],
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::SpaceAround,
        align_items: AlignItems::Center,
        ..default()
    }
}

/// Just shows the game time element. This consists of a watch icon, and text with the time running
#[derive(Component)]
#[require(Node = game_time_node())]
pub struct GameTimeDisplay;
fn game_time_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        grid_row: GridPlacement::start(2),
        grid_column: GridPlacement::start_end(10, 11),
        justify_self: JustifySelf::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

#[derive(Component)]
#[require(Text)]
pub struct GameTimeText;

pub struct StatDisplayPlugin<SC: StatComponent> {
    _mark: PhantomData<SC>,
}
impl<SC: StatComponent + DisplayableStat> StatDisplayPlugin<SC> {
    fn new() -> Self {
        Self { _mark: PhantomData }
    }
}

impl<SC: StatComponent + DisplayableStat> Plugin for StatDisplayPlugin<SC> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (spawn_individual_stat_hud_elements::<SC>,)
                    .run_if(in_state(AppState::InGame))
                    .run_if(|q_display: Query<&IndividualStatDisplay<SC>>| q_display.is_empty()),
                update_individual_stat_component::<SC>.run_if(in_state(AppState::InGame)),
            ),
        );
    }
}

#[derive(Component, Debug, Clone, Reflect)]
#[require(Node = slot_display_node())]
pub struct UpgradeSlotDisplay {
    pub weapons: [(Entity, Option<WeaponKind>); 5],
    pub stats: [(Entity, Option<StatUpgradeKind>); 5],
}
fn slot_display_node() -> Node {
    Node {
        height: Val::Percent(100.0),
        width: Val::Percent(100.0),
        justify_content: JustifyContent::SpaceEvenly,
        align_items: AlignItems::Center,
        flex_wrap: FlexWrap::Wrap,
        grid_template_rows: vec![RepeatedGridTrack::percent(2, 50.0)],
        grid_template_columns: vec![RepeatedGridTrack::percent(5, 20.0)],
        grid_column: GridPlacement::start_end(1, 7),
        grid_row: GridPlacement::start_end(16, 21),
        display: Display::Grid,
        ..default()
    }
}

impl UpgradeSlotDisplay {
    pub fn spawn(commands: &mut Commands, assets: &Res<AssetServer>) -> Entity {
        let display_ent = commands.spawn(slot_display_node()).id();
        let slot_image: Handle<Image> = assets.load("ui/upgrade_slot.png");
        let weapons = (0..5)
            .into_iter()
            .map(|_i| {
                let weapon_slot = commands
                    .spawn((
                        UpgradeSlot,
                        ImageNode::from(slot_image.clone())
                            .with_color(Color::srgba(1.0, 1.0, 1.0, 0.2)),
                        ChildOf(display_ent),
                    ))
                    .id();
                (weapon_slot, None)
            })
            .collect::<Vec<(Entity, Option<WeaponKind>)>>();

        let upgrades = (0..5)
            .into_iter()
            .map(|_i| {
                let upgrade_slot = commands
                    .spawn((
                        UpgradeSlot,
                        ImageNode::from(slot_image.clone())
                            .with_color(Color::srgba(1.0, 1.0, 1.0, 0.2)),
                        ChildOf(display_ent),
                    ))
                    .id();
                (upgrade_slot, None)
            })
            .collect::<Vec<(Entity, Option<StatUpgradeKind>)>>();
        let boxed_weapons: Box<[(Entity, Option<WeaponKind>); 5]> =
            weapons.into_boxed_slice().try_into().unwrap();
        let boxed_upgrades: Box<[(Entity, Option<StatUpgradeKind>); 5]> =
            upgrades.into_boxed_slice().try_into().unwrap();

        let display_box = UpgradeSlotDisplay {
            weapons: *boxed_weapons,
            stats: *boxed_upgrades,
        };
        commands.entity(display_ent).insert(display_box);
        display_ent
    }
}

#[derive(Component, Debug, Clone)]
#[require(Node = slot_node())]
pub struct UpgradeSlot;
fn slot_node() -> Node {
    Node {
        height: Val::Percent(100.0),
        width: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

#[derive(Component)]
#[require(Node = xp_bar_holder_node())]
pub struct XPBar;
fn xp_bar_holder_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        grid_column: GridPlacement::start_end(1, 21),
        grid_row: GridPlacement::start(1),
        ..default()
    }
}

#[derive(Component)]
#[require(Node = xp_bar_node(100.0, 100.0))]
pub struct XPBarBackground;

#[derive(Component)]
#[require(Node = xp_bar_node(0.0, 100.0))]
pub struct XPBarForeground;

fn xp_bar_node(width: f32, height: f32) -> Node {
    Node {
        width: Val::Percent(width),
        height: Val::Percent(height),
        align_items: AlignItems::FlexEnd,
        ..default()
    }
}

#[derive(Component)]
#[require(Node = health_bar_holder_node())]
pub struct HealthBar {
    /// Checked against the player's actual health and steadily brought to the same value if not equal
    displayed_pct: f32,
}
fn health_bar_holder_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(30.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        grid_column: GridPlacement::start_end(8, 13),
        grid_row: GridPlacement::start_end(16, 21),
        ..default()
    }
}

#[derive(Component)]
#[require(Node = hp_bar_node(100.0))]
pub struct HealthBarBackground;
fn hp_bar_node(height: f32) -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(height),
        ..default()
    }
}
#[derive(Component)]
#[require(Node = hp_bar_node(100.0))]
pub struct HealthBarForeground;

/// The outer node that holds all of the individual stats, except for the ones
/// that are kept in more prominent bars, like health and xp
#[derive(Component, Debug, Clone, Reflect)]
#[require(BackgroundColor = BackgroundColor(Color::srgba(1.0, 0.8, 0.8, 0.2)))]
pub struct StatDisplayContainer {
    kind: StatDisplayContainerKind,
}
#[derive(Debug, Clone, Reflect)]
pub enum StatDisplayContainerKind {
    PlayerHud,
    NonPlayerSummary,
}
fn stat_display_container(kind: StatDisplayContainerKind) -> Node {
    let mut node = Node::default();

    match kind {
        StatDisplayContainerKind::PlayerHud => {
            node.grid_auto_flow = GridAutoFlow::Column;
            node.grid_template_rows = vec![RepeatedGridTrack::percent(20, 5.0)];
            node.grid_template_columns = vec![RepeatedGridTrack::percent(1, 100.0)];
            node.width = Val::Percent(100.0);
            node.height = Val::Percent(100.0);
            node.grid_row = GridPlacement::start_end(4, 21);
            node.grid_column = GridPlacement::start_end(19, 21);
            node.flex_direction = FlexDirection::Column;
        }
        StatDisplayContainerKind::NonPlayerSummary => {
            node.grid_template_columns = vec![RepeatedGridTrack::percent(2, 50.0)];
        }
    }
    node
}

impl WeaponKind {
    fn to_icon_path(&self) -> Option<&str> {
        match self {
            WeaponKind::DiceGuard => Some("dice_guard"),
            WeaponKind::ThrowHands => Some("throw_hands"),
            _ => None,
        }
    }
}

/// Added to give the player a snapshot of their various stats.
/// This isn't used for things like XP or HP, which get their own
/// bars.
#[derive(Component)]
#[require(Node = stat_display_container_node(StatDisplayContainerKind::PlayerHud))]
pub struct IndividualStatDisplay<C> {
    pub stat_kind: StatKind,
    pub stat_value: C,
}
fn stat_display_container_node(kind: StatDisplayContainerKind) -> Node {
    let mut node = Node::default();
    match kind {
        StatDisplayContainerKind::PlayerHud => {
            node.height = Val::Percent(5.0);
            node.width = Val::Percent(100.0);
            node.justify_content = JustifyContent::SpaceEvenly;
            node.align_items = AlignItems::Center;
        }
        _ => {}
    }
    node
}

#[derive(Component)]
pub struct StatDisplayIcon;
impl StatDisplayIcon {
    fn path_from_stat_kind(sk: &StatKind) -> Option<&str> {
        match *sk {
            StatKind::Armor => Some("armor"),
            StatKind::AttackRange => Some("attack_range"),
            StatKind::CDR => Some("cooldown_rate"),
            StatKind::CritChance => Some("crit_chance"),
            StatKind::CritDamage => Some("crit_damage"),
            StatKind::Damage => Some("damage"),
            StatKind::EffDuration => Some("effect_duration"),
            StatKind::EffSize => Some("effect_size"),
            StatKind::Evasion => Some("evasion"),
            StatKind::Health => Some("max_health"),
            StatKind::HealthRegen => Some("health_regen"),
            StatKind::LifeSteal => Some("lifesteal"),
            StatKind::Luck => Some("luck"),
            StatKind::MS => Some("move_speed"),
            StatKind::PickupR => Some("pickup_radius"),
            StatKind::ProjBounces => Some("projectile_bounces"),
            StatKind::ProjCount => Some("projectile_count"),
            StatKind::ProjSpeed => Some("projectile_speed"),
            StatKind::Shield => Some("shield"),
            StatKind::Thorns => Some("thorns"),
            StatKind::XPGain => Some("xp_gain"),
        }
    }
}
#[derive(Component)]
#[require(Text)]
pub struct StatDisplayText;

fn spawn_individual_stat_hud_elements<C: StatComponent + DisplayableStat>(
    mut commands: Commands,
    assets: Res<AssetServer>,
    q_display_container: Single<Entity, With<StatDisplayContainer>>,
    q_component: Query<
        (Entity, &C),
        (
            With<C>,
            With<Player>,
            Or<(With<SinglePlayer>, With<Controlled>)>,
        ),
    >,
) {
    for (ent, comp) in &q_component {
        let stat_kind = comp.stat_kind();
        let stat_value = *comp;
        let display_container = commands
            .spawn((
                ChildOf(*q_display_container),
                IndividualStatDisplay {
                    stat_kind,
                    stat_value,
                },
            ))
            .id();

        let icon_name = StatDisplayIcon::path_from_stat_kind(&stat_kind);
        if let Some(name) = icon_name {
            let path = format!("ui/stat_icons/{}.png", name);
            let handle: Handle<Image> = assets.load(path);
            // Spawn the icon and the text for this element as well
            commands.spawn((
                ChildOf(display_container),
                StatDisplayIcon,
                ImageNode::from(handle),
            ));
        }
        commands.spawn((
            ChildOf(display_container),
            StatDisplayText,
            Text::from(format!("{:.1}", stat_value.display_value())),
        ));
    }
}

fn spawn_hud_container(mut commands: Commands, assets: Res<AssetServer>) {
    let outer_ent = commands
        .spawn((OuterHudContainer, DespawnOnExit(AppState::InGame)))
        .id();

    // The game clock
    let game_clock_container = commands.spawn((GameTimeDisplay, ChildOf(outer_ent))).id();
    let game_clock_text = commands
        .spawn((GameTimeText, ChildOf(game_clock_container)))
        .id();

    // XP Bar
    let xp_bar = commands.spawn((XPBar, ChildOf(outer_ent))).id();
    let xp_texture: Handle<Image> = assets.load("ui/health_bar_texture.png");
    let xp_background_node =
        ImageNode::from(xp_texture.clone()).with_color(Color::srgb(0.2, 0.2, 0.2));
    let xp_bg = commands
        .spawn((XPBarBackground, ChildOf(xp_bar), xp_background_node))
        .id();

    let xp_foreground_node = ImageNode::from(xp_texture).with_color(Color::srgb(0.3, 0.3, 0.9));
    commands.spawn((XPBarForeground, ChildOf(xp_bg), xp_foreground_node));

    // HP Bar
    let hp = commands
        .spawn((HealthBar { displayed_pct: 1.0 }, ChildOf(outer_ent)))
        .id();

    let hp_texture: Handle<Image> = assets.load("ui/health_bar_texture.png");

    let background_node =
        ImageNode::from(hp_texture.clone()).with_color(Color::srgb(0.2, 0.2, 0.2));
    let bg = commands
        .spawn((HealthBarBackground, ChildOf(hp), background_node))
        .id();

    let foreground_node = ImageNode::from(hp_texture).with_color(Color::srgb(1.0, 0.0, 0.0));
    commands.spawn((HealthBarForeground, ChildOf(bg), foreground_node));
    commands.spawn((
        StatDisplayContainer {
            kind: StatDisplayContainerKind::PlayerHud,
        },
        stat_display_container(StatDisplayContainerKind::PlayerHud),
        ChildOf(outer_ent),
    ));
    // Upgrade slots display
    let slots = UpgradeSlotDisplay::spawn(&mut commands, &assets);
    commands.entity(slots).insert(ChildOf(outer_ent));
}

fn update_health_bar(
    mut q_bar: Single<&mut Node, With<HealthBarForeground>>,
    q_player: Query<
        &Health,
        (
            With<Player>,
            Changed<Health>,
            Or<(With<SinglePlayer>, With<Controlled>)>,
        ),
    >,
) {
    for hp in &q_player {
        let pct = hp.current / hp.max();
        q_bar.width = Val::Percent(pct * 100.0)
    }
}

fn update_xp_bar(mut q_bar: Single<&mut Node, With<XPBarForeground>>, q_xp: Single<&LevelManager>) {
    let pct = (q_xp.c_xp - q_xp.prev_max) / (q_xp.next_max - q_xp.prev_max);
    q_bar.width = Val::Percent(pct * 100.0)
}

fn update_individual_stat_component<C: StatComponent + DisplayableStat>(
    mut q_container: Single<(&Children, &mut IndividualStatDisplay<C>)>,
    mut q_text: Query<&mut Text, With<StatDisplayText>>,
    q_changed: Query<
        &C,
        (
            Changed<C>,
            With<Player>,
            Or<(With<SinglePlayer>, With<Controlled>)>,
        ),
    >,
) {
    for stat in &q_changed {
        q_container.1.stat_value = *stat;
        for child in q_container.0 {
            if let Ok(mut t) = q_text.get_mut(*child) {
                t.0 = format!("{:.1}", stat.display_value());
            }
        }
    }
}

pub fn update_game_clock_display(
    time: Res<InGameTime>,
    mut q_text: Single<&mut Text, With<GameTimeText>>,
) {
    let mins = (time.elapsed_secs() / 60.0).floor();
    let secs = (time.elapsed_secs() - (60.0 * mins)).floor();
    q_text.0 = format!("{}:{:02}", mins, secs)
}

pub fn toggle_hud(mut q_screen: Single<&mut Visibility, With<OuterHudContainer>>) {
    q_screen.toggle_inherited_hidden();
}

fn hud_not_shown(q_screen: Single<&Visibility, With<OuterHudContainer>>) -> bool {
    match *q_screen {
        Visibility::Hidden => true,
        _ => false,
    }
}

fn update_upgrade_slot_display(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut q_slot_holder: Single<&mut UpgradeSlotDisplay>,
    q_player: Option<
        Single<
            &PlayerUpgradeSlots,
            (
                Changed<PlayerUpgradeSlots>,
                With<Player>,
                Or<(With<SinglePlayer>, With<Predicted>)>,
            ),
        >,
    >,
) {
    if let Some(upgrades) = q_player {
        // Weapons first
        let c_weapon_slots = q_slot_holder
            .weapons
            .iter()
            .filter_map(|(e, m_weapon)| *m_weapon)
            .collect::<Vec<WeaponKind>>();

        // Check to see if the player weapons are present
        let weapons_to_add = upgrades
            .weapons
            .keys()
            .filter_map(|w| {
                if c_weapon_slots.contains(w) {
                    None
                } else {
                    Some(*w)
                }
            })
            .collect::<Vec<WeaponKind>>();

        for w in weapons_to_add {
            let next = q_slot_holder
                .weapons
                .iter()
                .position(|(e, m_w)| m_w.is_none())
                .expect(
                    "This should not have been an offered upgrade if we don't have a free slot",
                );
            let mut entry = q_slot_holder.weapons.get_mut(next).unwrap();
            let weapon_icon_path = w.to_icon_path();
            if let Some(name) = weapon_icon_path {
                let path = format!("ui/weapon_icons/{}.png", name);
                let w_icon: Handle<Image> = assets.load(path);
                commands.entity(entry.0).with_children(|parent| {
                    parent.spawn(ImageNode::from(w_icon));
                });
            }
            entry.1 = Some(w);
        }

        // now stats
        let c_stat_slots = q_slot_holder
            .stats
            .iter()
            .filter_map(|(e, m_stat)| *m_stat)
            .collect::<Vec<StatUpgradeKind>>();

        // Check to see if the player weapons are present
        let stats_to_add = upgrades
            .stats
            .keys()
            .filter_map(|s| {
                if c_stat_slots.contains(s) {
                    None
                } else {
                    Some(*s)
                }
            })
            .collect::<Vec<StatUpgradeKind>>();

        for s in stats_to_add {
            let next = q_slot_holder
                .stats
                .iter()
                .position(|(e, m_s)| m_s.is_none())
                .expect(
                    "This should not have been an offered upgrade if we don't have a free slot",
                );
            let mut entry = q_slot_holder.stats.get_mut(next).unwrap();
            let stat: StatKind = s.into();
            let icon_name = StatDisplayIcon::path_from_stat_kind(&stat);
            if let Some(name) = icon_name {
                let path = format!("ui/stat_icons/{}.png", name);
                let handle: Handle<Image> = assets.load(path);
                commands.entity(entry.0).with_children(|parent| {
                    parent.spawn(ImageNode::from(handle));
                });
            }
            entry.1 = Some(s);
        }
    }
}
