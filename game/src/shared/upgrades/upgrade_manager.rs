use super::*;
use bevy::prelude::*;
use rand::seq::IndexedRandom;
use strum::IntoEnumIterator;

#[derive(Resource)]
pub struct UpgradeManager;

impl UpgradeManager {
    pub fn generate_upgrade_options(c_upgrades: &PlayerUpgradeSlots) -> UpgradeOptions {
        // Gather the set of currently taken upgrades in each case
        let c_weapons_iter = c_upgrades.weapons.keys();
        let weapons_len_check = c_weapons_iter.len() < c_upgrades.weapon_limit;
        let mut available_upgrades = c_weapons_iter
            .map(|w| UpgradeKind::UpgradeWeapon(*w))
            .collect::<Vec<UpgradeKind>>();
        if weapons_len_check {
            // We can add any other weapon but the ones we already have
            let mut rem_weapons = WeaponKind::iter()
                .filter_map(|w| {
                    if c_upgrades.weapons.get(&w).is_some() {
                        None
                    } else {
                        Some(UpgradeKind::AddWeapon(w))
                    }
                })
                .collect::<Vec<UpgradeKind>>();

            available_upgrades.append(&mut rem_weapons);
        }
        // Do the same for stats
        let c_stats_iter = c_upgrades.stats.keys();
        let mut stats_upgrades = if c_stats_iter.len() == c_upgrades.stats_limit {
            c_stats_iter
                .map(|stat_upgrade| UpgradeKind::UpgradePlayerStat(*stat_upgrade))
                .collect::<Vec<UpgradeKind>>()
        } else {
            StatUpgradeKind::iter()
                .map(|stat_upgrade| UpgradeKind::UpgradePlayerStat(stat_upgrade))
                .collect::<Vec<UpgradeKind>>()
        };
        available_upgrades.append(&mut stats_upgrades);

        // Pick randomly from these upgrades
        let upgrades: Vec<_> = available_upgrades
            .choose_multiple(&mut rand::rng(), 3)
            .collect();

        // Let's roll
        let options = upgrades
            .into_iter()
            .map(|uk| {
                let rarity: UpgradeRarity = rand::random();

                let level = match *uk {
                    UpgradeKind::AddWeapon(_w) => 1,
                    UpgradeKind::UpgradeWeapon(w) => c_upgrades.weapons.get(&w).unwrap() + 1,
                    UpgradeKind::UpgradePlayerStat(upgrade_stat) => {
                        if let Some(v) = c_upgrades.stats.get(&upgrade_stat) {
                            v + 1
                        } else {
                            1
                        }
                    }
                };
                Upgrade::from_data(*uk, rarity, level)
            })
            .collect::<Vec<Upgrade>>();

        let boxed_options: Box<[Upgrade; 3]> = options.into_boxed_slice().try_into().unwrap();
        UpgradeOptions {
            options: *boxed_options,
            selected: None,
        }
    }
}

pub fn add_upgrade_manager(mut commands: Commands) {
    commands.insert_resource(UpgradeManager);
}
