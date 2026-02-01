#![cfg(feature = "dev")]
use bevy::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use ron::ser::PrettyConfig;

use super::*;

pub struct UpgradeEditorPlugin;
impl Plugin for UpgradeEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UpgradeEditor>()
            .add_plugins(ResourceInspectorPlugin::<UpgradeEditor>::new())
            .add_systems(Update, editor_on_save);
    }
}

/// This resource allows for user input to construct values for upgrades.
///
/// The idea right now is that upgrades, depending on the type, will have different
/// possible rolls that are allowed based on the rarity of the upgrade. Those upgrades will
/// in turn be applied based on the value of the lookup tables at the time that the upgrade
/// is taken
#[derive(Reflect, Resource)]
pub struct UpgradeEditor {
    save: bool,
    c_table: RawUpgradeTable,
}
impl Default for UpgradeEditor {
    fn default() -> Self {
        Self {
            save: false,
            c_table: crate::utils::read_ron("assets/upgrades/table_tmp.ron".into()),
        }
    }
}

impl UpgradeEditor {
    fn save(table: RawUpgradeTable) {
        let ron_string = ron::ser::to_string_pretty(&table, PrettyConfig::new())
            .expect("Failed to serialize stats path");
        let _write = std::fs::write("assets/upgrades/table_tmp.ron", ron_string);
    }
}

pub fn editor_on_save(mut editor: ResMut<UpgradeEditor>) {
    if editor.save {
        UpgradeEditor::save(editor.c_table.clone());
        editor.save = false;
    }
}
