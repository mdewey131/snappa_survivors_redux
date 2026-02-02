use std::marker::PhantomData;

use crate::shared::{
    game_kinds::{DefaultClientFilter, SinglePlayer},
    players::Player,
    states::*,
    stats::{
        components::{Health, StatComponent},
        *,
    },
};
use bevy::prelude::*;
use lightyear::prelude::Controlled;

pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_hud_container)
            .add_systems(
                Update,
                update_health_bar.run_if(in_state(InGameState::InGame)),
            );
    }
}

#[derive(Component, Debug)]
#[require(Node = outer_hud_node())]
pub struct OuterHudContainer;
fn outer_hud_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::SpaceAround,
        align_items: AlignItems::FlexEnd,
        ..default()
    }
}

pub struct StatDisplayPlugin<SC: StatComponent> {
    _mark: PhantomData<SC>,
}

impl<SC: StatComponent> Plugin for StatDisplayPlugin<SC> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_hud_elements_on_component_add::<SC>)
                .run_if(in_state(AppState::InGame))
                .run_if(|q_display: Query<&IndividualStatDisplay<SC>>| q_display.is_empty()),
        );
    }
}

fn main_hud_component_node(width: f32) -> Node {
    Node {
        height: Val::Percent(10.0),
        width: Val::Percent(width),
        ..default()
    }
}

#[derive(Component)]
#[require(Node = main_hud_component_node(30.0))]
pub struct XPBar;

#[derive(Component)]
#[require(Node = main_hud_component_node(30.0))]
pub struct HealthBar {
    /// Checked against the player's actual health and steadily brought to the same value if not equal
    displayed_pct: f32,
}

#[derive(Component)]
#[require(Node = hp_bar_node(30.0))]
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
#[require(Node = main_hud_component_node(30.0))]
pub struct StatDisplayContainer;

/// Added to give the player a snapshot of their various stats.
/// This isn't used for things like XP or HP, which get their own
/// bars.
#[derive(Component)]
pub struct IndividualStatDisplay<C> {
    pub target_player: Entity,
    pub stat_kind: StatKind,
    pub stat_value: C,
}

fn spawn_hud_elements_on_component_add<C: StatComponent>(
    mut commands: Commands,
    q_display_container: Single<Entity, With<StatDisplayContainer>>,
    q_added: Query<(Entity, &C), (Added<C>, With<Player>, DefaultClientFilter)>,
) {
    for (ent, comp) in &q_added {
        let stat_kind = comp.stat_kind();
        let stat_value = *comp;
        commands.spawn((
            ChildOf(*q_display_container),
            IndividualStatDisplay {
                target_player: ent,
                stat_kind,
                stat_value,
            },
        ));
    }
}

fn spawn_hud_container(mut commands: Commands, assets: Res<AssetServer>) {
    let outer_ent = commands.spawn((OuterHudContainer)).id();

    commands.spawn((XPBar, ChildOf(outer_ent)));
    let hp = commands
        .spawn((HealthBar { displayed_pct: 1.0 }, ChildOf(outer_ent)))
        .id();

    let texture: Handle<Image> = assets.load("ui/health_bar_texture.png");

    let background_node = ImageNode::from(texture.clone()).with_color(Color::srgb(0.2, 0.2, 0.2));
    let bg = commands
        .spawn((HealthBarBackground, ChildOf(hp), background_node))
        .id();

    let foreground_node = ImageNode::from(texture).with_color(Color::srgb(1.0, 0.0, 0.0));
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
