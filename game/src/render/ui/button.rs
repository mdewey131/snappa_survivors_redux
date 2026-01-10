use bevy::{ecs::system::SystemId, platform::collections::HashMap, prelude::*};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct ButtonSystems(pub HashMap<String, SystemId>);

#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct ButtonReleased {
    pub entity: Entity,
}

#[derive(Clone, Copy, Debug)]
pub enum GameButtonOnRelease {
    TriggerSystem(SystemId),
    // Triggers the `ButtonReleased` event
    EventTrigger,
}

#[derive(Component, Debug, Clone)]
#[require(Button, Node = game_button_node())]
pub struct GameButton {
    // Triggers with this button happen on release, so that the player can cancel a click if desired
    pub on_release: GameButtonOnRelease,
    pub ready: bool,
    click_hold_timer: Timer,
}

fn game_button_node() -> Node {
    Node {
        height: Val::Percent(50.0),
        width: Val::Percent(50.0),
        ..default()
    }
}

impl GameButton {
    pub fn new(on_release: GameButtonOnRelease) -> Self {
        Self {
            on_release,
            ready: false,
            click_hold_timer: Timer::from_seconds(0.05, TimerMode::Once),
        }
    }

    pub fn spawn(
        self,
        commands: &mut Commands,
        assets: &Res<AssetServer>,
        style: GameButtonStyle,
    ) -> Entity {
        let self_id = commands.spawn_empty().id();

        style.add_style_components(self_id, commands, &assets);

        commands.entity(self_id).insert((self,));

        self_id
    }

    fn on_release(&self, commands: &mut Commands, self_ent: Entity) {
        match self.on_release {
            GameButtonOnRelease::EventTrigger => {
                commands.trigger(ButtonReleased { entity: self_ent });
            }
            GameButtonOnRelease::TriggerSystem(sys) => {
                commands.run_system(sys);
            }
        }
    }
}

#[derive(Default)]
pub enum GameButtonImage {
    #[default]
    Default,
    Custom(Handle<Image>),
}

/// We want the buttons in the game to have a specific styling, so we make this type
/// to describe how you can calibrate the buttons within our confines (at least, that's the idea)
#[derive(Default)]
pub struct GameButtonStyle {
    pub image: GameButtonImage,
    pub node: Node,
    pub color: Option<Color>,
    pub text: Option<Text>,
    /// For now, always uniform
    pub border: Option<(Val, Color)>,
}

impl GameButtonStyle {
    pub fn new(im: GameButtonImage) -> Self {
        Self {
            image: im,
            node: Self::default_node(),
            color: None,
            text: None,
            border: None,
        }
    }
    pub fn with_color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }

    pub fn with_text(mut self, s: String) -> Self {
        self.text = Some(Text::new(s));
        self
    }

    pub fn with_border(mut self, size: Val, col: Color) -> Self {
        self.border = Some((size, col));
        self
    }

    pub fn with_size(mut self, height: Val, width: Val) -> Self {
        self.node.height = height;
        self.node.width = width;
        self
    }

    /// The main function used to turn the style into a button
    pub fn add_style_components(
        self,
        to: Entity,
        commands: &mut Commands,
        assets: &Res<AssetServer>,
    ) {
        // The mandatory things
        let mut im_node = match self.image {
            GameButtonImage::Custom(i) => ImageNode::new(i),
            GameButtonImage::Default => ImageNode::new(assets.load("ui/button.png")),
        };

        if let Some(c) = self.color {
            im_node = im_node.with_color(c);
        }

        commands.entity(to).insert((im_node, self.node));

        // The optional things
        if let Some(text) = self.text {
            commands.entity(to).with_child(text);
        }

        if let Some((size, col)) = self.border {
            commands
                .entity(to)
                .insert((BorderRadius::all(size), BorderColor::all(col)));
        }
    }

    /// Temporary while I figure out how I want to do node styling
    pub fn default_node() -> Node {
        Node {
            height: Val::Percent(50.0),
            width: Val::Percent(50.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        }
    }
}

pub struct GameButtonPlugin;

impl Plugin for GameButtonPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ButtonSystems(HashMap::new()))
            .add_systems(Update, (update,).chain());
    }
}

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut q_button: Query<(Entity, &mut GameButton, &Interaction)>,
) {
    for (b_ent, mut button, int) in &mut q_button {
        match *int {
            Interaction::Pressed => {
                button.click_hold_timer.tick(time.delta());
                if button.click_hold_timer.is_finished() {
                    button.ready = true
                }
            }
            Interaction::Hovered | Interaction::None => {
                if button.ready {
                    button.on_release(&mut commands, b_ent);
                }
                button.click_hold_timer.reset();
                button.ready = false;
            }
        }
    }
}
