use std::marker::PhantomData;

use crate::shared::{
    game_kinds::{DefaultClientFilter, SinglePlayer},
    players::Player,
    states::*,
    stats::{components::*, xp::LevelManager, *},
};
use bevy::prelude::*;
use lightyear::prelude::Controlled;

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
                (update_health_bar, update_xp_bar, update_game_clock_display)
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
#[derive(Component)]
#[require(Node = stat_display_container(100.0))]
pub struct StatDisplayContainer;
fn stat_display_container(width: f32) -> Node {
    Node {
        display: Display::Grid,
        height: Val::Percent(100.0),
        width: Val::Percent(width),
        justify_content: JustifyContent::SpaceEvenly,
        align_items: AlignItems::Center,
        flex_wrap: FlexWrap::Wrap,
        grid_template_columns: vec![RepeatedGridTrack::percent(2, 50.0)],
        grid_row: GridPlacement::start_end(14, 21),
        grid_column: GridPlacement::start_end(14, 21),
        ..default()
    }
}

/// Added to give the player a snapshot of their various stats.
/// This isn't used for things like XP or HP, which get their own
/// bars.
#[derive(Component)]
#[require(Node = stat_display_container_node())]
pub struct IndividualStatDisplay<C> {
    pub stat_kind: StatKind,
    pub stat_value: C,
}
fn stat_display_container_node() -> Node {
    Node {
        height: Val::Percent(10.0),
        width: Val::Percent(50.0),
        justify_content: JustifyContent::SpaceEvenly,
        align_items: AlignItems::Center,
        ..default()
    }
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
            StatKind::EffDuration => Some("effect_duration"),
            _ => None,
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
    let outer_ent = commands.spawn((OuterHudContainer)).id();

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
    commands.spawn((StatDisplayContainer, ChildOf(outer_ent)));
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
    let pct = (q_xp.c_xp - q_xp.prev_max) / q_xp.next_max;
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
