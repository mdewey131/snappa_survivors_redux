use super::RawStatsList;
use crate::{
    shared::{enemies::EnemyKind, players::CharacterKind, weapons::WeaponKind},
    utils::AssetFolder,
};
use bevy::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use ron::ser::PrettyConfig;

/// A plugin on the stats editor, only to be used in cases where we're in a dev environment!
#[cfg(feature = "dev")]
pub struct StatsEditorPlugin;

#[cfg(feature = "dev")]
impl Plugin for StatsEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ResourceInspectorPlugin::<StatsEditor>::default())
            .init_resource::<StatsEditor>()
            .add_systems(Update, (save_changes_to_stats, load_new_stats).chain());
    }
}

/// Used to import and edit base stats, allowing one to save
/// the new stats and affect subsequent spawns
#[derive(Resource, Reflect, Default)]
pub struct StatsEditor {
    /// The user selected value of the character kind
    pub c_char: CharacterKind,
    pub save_char: bool,
    pub c_char_stats: Option<RawStatsList>,
    pub c_enemy: EnemyKind,
    pub save_enemy: bool,
    pub c_enemy_stats: Option<RawStatsList>,
    pub c_weapon: WeaponKind,
    pub save_weapon: bool,
    pub c_weapon_stats: Option<RawStatsList>,
    /// Used for change detection
    #[reflect(ignore)]
    p_char: Option<CharacterKind>,
    /// Used for change detection
    #[reflect(ignore)]
    p_enemy: Option<EnemyKind>,
    /// Used for change detection
    #[reflect(ignore)]
    p_weapon: Option<WeaponKind>,
}

impl StatsEditor {
    fn save_procedure(list: &RawStatsList, to_folder: impl Into<AssetFolder>) {
        let to_write = list.clone();
        let asset_folder: AssetFolder = to_folder.into();
        let stats_path = format!("assets/{}", asset_folder.to_path("stats.ron".into()));
        let ron_string = ron::ser::to_string_pretty(&to_write, PrettyConfig::new())
            .expect("Failed to serialize stats path");
        let _write = std::fs::write(stats_path, ron_string);
    }

    pub fn save_enemy(&mut self) {
        if let Some(ref list) = self.c_enemy_stats {
            Self::save_procedure(list, self.c_enemy);
        }
        self.save_enemy = false;
    }

    pub fn save_char(&mut self) {
        if let Some(ref list) = self.c_char_stats {
            Self::save_procedure(list, self.c_char);
        }
        self.save_char = false;
    }

    pub fn save_weapon(&mut self) {
        if let Some(ref list) = self.c_weapon_stats {
            Self::save_procedure(list, self.c_weapon);
        }
        self.save_weapon = false;
    }
}

fn load_new_stats(mut editor: ResMut<StatsEditor>) {
    let load_char = match editor.p_char {
        Some(c) => editor.c_char != c,
        None => true,
    };
    if load_char {
        let stats = RawStatsList::import_stats(editor.c_char);
        editor.c_char_stats = Some(stats);
        editor.p_char = Some(editor.c_char);
    }

    let load_enemy = match editor.p_enemy {
        Some(e) => editor.c_enemy != e,
        None => true,
    };

    if load_enemy {
        let stats = RawStatsList::import_stats(editor.c_enemy);
        editor.c_enemy_stats = Some(stats);
        editor.p_enemy = Some(editor.c_enemy);
    }

    let load_weapon = match editor.p_weapon {
        Some(e) => editor.c_weapon != e,
        None => true,
    };

    if load_weapon {
        let stats = RawStatsList::import_stats(editor.c_weapon);
        editor.c_weapon_stats = Some(stats);
        editor.p_weapon = Some(editor.c_weapon);
    }
}

fn save_changes_to_stats(mut editor: ResMut<StatsEditor>) {
    if editor.save_char {
        editor.save_char();
    }
    if editor.save_enemy {
        editor.save_enemy();
    }
    if editor.save_weapon {
        editor.save_weapon();
    }
}
