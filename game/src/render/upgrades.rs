use super::ui::button::*;
use crate::{
    render::ui::FadeEffect,
    shared::{
        despawn_timer::DespawnTimer,
        game_kinds::SinglePlayer,
        players::Player,
        states::InGameState,
        upgrades::{Upgrade, UpgradeOptions, UpgradeSelectionMessage},
    },
};
use bevy::{ecs::entity_disabling::Disabled, prelude::*};
use lightyear::prelude::Controlled;

pub struct UpgradeRenderPlugin;

impl Plugin for UpgradeRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InGameState::SelectingUpgrades),
            spawn_upgrade_screen,
        )
        .add_systems(
            Update,
            spawn_upgrade_buttons
                .run_if(in_state(InGameState::SelectingUpgrades))
                .run_if(|q_button: Query<&UpgradeButton>| q_button.is_empty()),
        )
        .add_observer(on_upgrade_choice_selection);
    }
}

#[derive(Component)]
#[require(Node = upgrade_screen_node())]
pub struct UpgradeScreen;
fn upgrade_screen_node() -> Node {
    Node {
        height: Val::Percent(100.0),
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        ..default()
    }
}

#[derive(Component)]
#[require(Button, Node = upgrade_button_node())]
pub struct UpgradeButton {
    pub upgrade_index: usize,
}
fn upgrade_button_node() -> Node {
    Node {
        width: Val::Percent(50.0),
        height: Val::Percent(80.0),
        justify_content: JustifyContent::Center,
        flex_direction: FlexDirection::Column,
        ..default()
    }
}
#[derive(Component)]
#[require(Text, Node = upgrade_title_text())]
pub struct UpgradeButtonTitle;
fn upgrade_title_text() -> Node {
    Node {
        height: Val::Percent(20.0),
        width: Val::Percent(90.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

#[derive(Component)]
#[require(Text, Node = upgrade_description_text())]
pub struct UpgradeButtonDescription;
fn upgrade_description_text() -> Node {
    Node {
        height: Val::Percent(20.0),
        width: Val::Percent(90.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

fn spawn_upgrade_screen(
    mut commands: Commands,
    q_player: Option<
        Single<&UpgradeOptions, (With<Player>, Or<(With<Controlled>, With<SinglePlayer>)>)>,
    >,
) {
    let screen = commands.spawn(UpgradeScreen).id();
    if let Some(player) = q_player {
        for (idx, option) in player.options.iter().enumerate() {
            let game_button = GameButton::new(GameButtonOnRelease::EventTrigger);
            commands
                .spawn((
                    ChildOf(screen),
                    UpgradeButton {
                        upgrade_index: idx,
                    },
                    game_button,
                ))
                .with_children(|p| {
                    p.spawn((UpgradeButtonTitle, Text::from(format!("{:?}", option.kind))));
                    p.spawn((
                        UpgradeButtonDescription,
                        Text::from(format!("{:?}, Level {}", option.rarity, option.level)),
                    ));
                });
        }
    }
}

/// Run as a backstop on the client, because the upgradeoptions might arrive later than the message to move in game
fn spawn_upgrade_buttons(
    mut commands: Commands,
    q_player: Single<&UpgradeOptions, (With<Player>, Or<(With<Controlled>, With<SinglePlayer>)>)>,
    q_screen: Single<Entity, With<UpgradeScreen>>,
) {
    for (idx, option) in q_player.options.iter().enumerate() {
        let game_button = GameButton::new(GameButtonOnRelease::EventTrigger);
        commands
            .spawn((
                ChildOf(*q_screen),
                UpgradeButton {
                    upgrade_index: idx,
                },
                game_button,
            ))
            .with_children(|p| {
                p.spawn((UpgradeButtonTitle, Text::from(format!("{:?}", option.kind))));
                p.spawn((
                    UpgradeButtonDescription,
                    Text::from(format!("{:?}, Level {}", option.rarity, option.level)),
                ));
            });
    }
}

fn on_upgrade_choice_selection(
    trigger: On<ButtonReleased>,
    mut commands: Commands,
    mut messages: MessageWriter<UpgradeSelectionMessage>,
    q_choice: Query<(Entity, &UpgradeButton)>,
    q_screen: Single<Entity, With<UpgradeScreen>>,
) {
    let selection = if let Ok((_e, button)) = q_choice.get(trigger.entity) {
        button.upgrade_index
    } else {
        return;
    };
    messages.write(UpgradeSelectionMessage(selection));
    commands.entity(*q_screen).insert((
        FadeEffect::fade_out(1.0, EaseFunction::Linear),
        DespawnTimer::new(1.0),
    ));
}
