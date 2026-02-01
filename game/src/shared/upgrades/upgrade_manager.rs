use std::path::Path;

use super::*;
use bevy::prelude::*;
use rand::{
    SeedableRng,
    rngs::{SmallRng, ThreadRng},
    seq::{IndexedRandom, IteratorRandom},
};
use strum::IntoEnumIterator;

/// Contains its own rng thread so that we can somewhat control the distribution for testing
#[derive(Resource)]
pub struct UpgradeManager {
    /// Recommended by the rng book for non-crypto purposes
    rng: SmallRng,
    seed: [u8; 32],
    pub table: UpgradeTable,
}

impl UpgradeManager {
    pub fn generate_upgrade_options(&mut self, c_upgrades: &PlayerUpgradeSlots) -> UpgradeOptions {
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
            .into_iter()
            .choose_multiple(&mut self.rng, 3)
            .into_iter()
            .collect();

        // Let's roll
        let options = upgrades
            .into_iter()
            .map(|uk| {
                let rarity: UpgradeRarity = rand::random();

                let level = match uk {
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
                self.make_upgrade(uk, rarity, level)
            })
            .collect::<Vec<Upgrade>>();

        let boxed_options: Box<[Upgrade; 3]> = options.into_boxed_slice().try_into().unwrap();
        UpgradeOptions {
            options: *boxed_options,
            selected: None,
        }
    }

    pub fn make_upgrade(&mut self, kind: UpgradeKind, rarity: UpgradeRarity, level: u8) -> Upgrade {
        let tmp_backup_reward = UpgradeReward::default();
        let table_entry = self.table.table.get(&kind);
        let rewards: Vec<UpgradeReward> = if table_entry.is_none() {
            vec![tmp_backup_reward]
        } else {
            match kind {
                UpgradeKind::AddWeapon(_w) => table_entry.unwrap().clone(),
                _ => {
                    let mut rewards_with_rolls = Vec::new();
                    for reward in table_entry.unwrap().into_iter() {
                        match reward {
                            UpgradeReward::StatUpgrade {
                                range: range,
                                kind: kind,
                                value: value,
                            } => {
                                let v = (&mut self.rng).random_range(range.min..range.max);
                                rewards_with_rolls.push(UpgradeReward::StatUpgrade {
                                    range: *range,
                                    kind: *kind,
                                    value: Some(v),
                                });
                            }
                            r => rewards_with_rolls.push(*r),
                        }
                    }

                    let chosen_rewards = rewards_with_rolls
                        .into_iter()
                        .choose_multiple(&mut self.rng, 1);

                    chosen_rewards
                }
            }
        };

        Upgrade {
            kind,
            rarity,
            level,
            rewards: rewards,
        }
    }
}

pub fn add_upgrade_manager(mut commands: Commands) {
    let seed = [0; 32];

    commands.insert_resource(UpgradeManager {
        rng: SmallRng::from_seed(seed),
        seed,
        table: UpgradeTable::new(),
    });
}

#[derive(Serialize, Deserialize, Reflect, Clone, Copy, Default, PartialEq, Debug)]
#[reflect(Default)]
pub struct StatUpgrade {
    min: f32,
    max: f32,
}

#[derive(Serialize, Deserialize, Reflect, Clone, Copy, PartialEq, Debug)]
#[reflect(Default)]
pub enum UpgradeReward {
    StatUpgrade {
        range: StatUpgrade,
        kind: StatUpgradeKind,
        #[reflect(ignore)]
        value: Option<f32>,
    },
    AddWeapon(WeaponKind),
}
impl Default for UpgradeReward {
    fn default() -> Self {
        Self::StatUpgrade {
            range: StatUpgrade { min: 0.0, max: 1.0 },
            kind: StatUpgradeKind::Armor,
            value: None,
        }
    }
}

pub struct UpgradeTable {
    table: HashMap<UpgradeKind, Vec<UpgradeReward>>,
}
impl UpgradeTable {
    pub fn new() -> Self {
        let mut upgrades = HashMap::new();
        let raw_table =
            crate::utils::read_ron::<RawUpgradeTable>("assets/upgrades/table_tmp.ron".into());
        for row in raw_table.0.into_iter() {
            upgrades.insert(row.kind, row.rewards);
        }
        Self { table: upgrades }
    }
}

#[derive(Serialize, Deserialize, Reflect, Clone, Default)]
#[reflect(Default)]
pub struct RawUpgradeTable(Vec<RawUpgradeTableRow>);

#[derive(Serialize, Deserialize, Reflect, Clone, Default)]
#[reflect(Default)]
pub struct RawUpgradeTableRow {
    kind: UpgradeKind,
    rewards: Vec<UpgradeReward>,
}
