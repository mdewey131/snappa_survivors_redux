use std::{
    rc::Rc,
    sync::{Arc, Mutex, Weak},
};

use bevy::{platform::collections::HashMap, prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

pub mod components;
pub mod editor;
pub mod plugins;
/*
pub mod relationships;
*/
pub mod xp;

use components::*;
use xp::LevelManager;

use crate::utils::AssetFolder;

/// The result of the inevitable "rewrite into an enum"
#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Reflect, Eq, PartialOrd, Ord,
)]
pub enum StatKind {
    AttackRange,
    Armor,
    CritChance,
    CritDamage,
    CDR,
    Damage,
    EffDuration,
    EffSize,
    Evasion,
    Health,
    HealthRegen,
    Luck,
    LifeSteal,
    MS,
    PickupR,
    ProjCount,
    ProjSpeed,
    Shield,
    Thorns,
    XPGain,
}

impl StatKind {
    fn to_component(&self, ec: &mut EntityCommands, input: f32) {
        match *self {
            StatKind::Armor => {
                ec.insert(Armor(input));
            }
            StatKind::AttackRange => {
                ec.insert(AttackRange(input));
            }
            StatKind::CDR => {
                ec.insert(CooldownRate(input));
            }
            StatKind::CritChance => {
                ec.insert(CritChance(input));
            }
            StatKind::CritDamage => {
                ec.insert(CritDamage(input));
            }
            StatKind::Damage => {
                ec.insert(Damage(input));
            }
            StatKind::EffDuration => {
                ec.insert(EffectDuration(input));
            }
            StatKind::EffSize => {
                ec.insert(EffectSize(input));
            }
            StatKind::Evasion => {
                ec.insert(Evasion(input));
            }
            StatKind::Health => {
                ec.insert(Health::new(input));
            }
            StatKind::Evasion => {
                ec.insert(Evasion(input));
            }
            StatKind::HealthRegen => {
                ec.insert(HealthRegen(input));
            }
            StatKind::LifeSteal => {
                ec.insert(LifeSteal(input));
            }
            StatKind::Luck => {
                ec.insert(Luck(input));
            }
            StatKind::MS => {
                ec.insert(MovementSpeed {
                    current: input,
                    cap: 600.0,
                });
            }
            StatKind::PickupR => {
                ec.insert(PickupRadius(input));
            }
            StatKind::ProjCount => {
                ec.insert(ProjectileCount(input));
            }
            StatKind::ProjSpeed => {
                ec.insert(ProjectileSpeed(input));
            }
            StatKind::Shield => {
                ec.insert(Shield(input));
            }
            StatKind::Thorns => {
                ec.insert(Thorns(input));
            }
            StatKind::XPGain => {
                ec.insert(XPGain(input));
            }
        }
    }
}

/// Holds references to the current modifier.
/// One limitation of this approach is that a stat hasn't changed until its
/// observed. In theory this isn't a problem, but it may introduce some overhead
/// when we're constantly in need of a mutable reference to self in order to get
/// the current value.
///
/// A potential solution to this problem is making a set of StatComponents
/// that are responsible for reading from the mutex at the end of every frame,
/// and can therefore be kept around as read-only copies of the stats that an
/// entity possesses
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Stat {
    // this value gets changed, which causes changes...
    pub base_value: f32,
    // by adding with its modifiers...
    pub modifiers: Vec<StatModifier>,
    // to equal the current value
    #[reflect(ignore)]
    current: Arc<Mutex<f32>>,
}

impl Stat {
    pub fn get_current(&mut self) -> Option<f32> {
        let modifier_total = self
            .modifiers
            .iter_mut()
            .map(|modifier| modifier.val(self.base_value))
            .sum::<f32>();
        if let Ok(mut guard) = self.current.lock() {
            *guard = self.base_value + modifier_total;
            Some((*guard))
        } else {
            None
        }
    }
    pub fn clone_current_weak(&self) -> Weak<Mutex<f32>> {
        let new_arc = self.current.clone();
        Arc::downgrade(&new_arc)
    }
}

#[derive(Component, Reflect, Debug)]
pub struct StatModifier {
    /// Holds a weak reference to the stat that it takes from
    #[reflect(ignore)]
    pub from_stat: Weak<Mutex<f32>>,
    /// How to combine with the stat to make the modifier
    pub method: StatModifierMethod,
    val: f32,
}

impl StatModifier {
    /// TODO: Make this value something other than 0
    pub fn new(from_stat: Weak<Mutex<f32>>, method: StatModifierMethod) -> Self {
        Self {
            from_stat,
            method,
            val: 0.0,
        }
    }
    fn val(&mut self, base_stat: f32) -> f32 {
        let stat_value = match self.from_stat.upgrade() {
            Some(arc) => match arc.lock() {
                Ok(mutex_guard) => *mutex_guard,
                Err(_) => 0.0,
            },
            None => 0.0,
        };
        match self.method {
            StatModifierMethod::FlatAdd => stat_value,
            StatModifierMethod::MultipliyWithBase { coefficient } => {
                ((base_stat * stat_value * coefficient) - base_stat)
            }
        }
    }
}

#[derive(Component, Reflect, Debug)]
pub enum StatModifierMethod {
    FlatAdd,
    MultipliyWithBase { coefficient: f32 },
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct StatList {
    pub list: HashMap<StatKind, Stat>,
}

impl StatList {
    fn new() -> Self {
        Self {
            list: HashMap::new(),
        }
    }
    fn get_current(&mut self, stat_kind: &StatKind) -> Option<f32> {
        let stat = self.list.get_mut(stat_kind);
        if let Some(s) = stat {
            if let Some(v) = s.get_current() {
                return Some(v);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
}

#[derive(Serialize, Deserialize, Reflect, Debug, Clone, Default)]
#[reflect(Default)]
pub struct RawStatsList(Vec<RawStatConstructor>);

#[derive(Serialize, Deserialize, Reflect, Debug, Clone)]
pub struct RawStatConstructor {
    kind: StatKind,
    val: f32,
}
impl Default for RawStatConstructor {
    fn default() -> Self {
        Self {
            kind: StatKind::Health,
            val: 50.0,
        }
    }
}

impl RawStatsList {
    pub fn import_stats(to_folder: impl Into<AssetFolder>) -> Self {
        let folder: AssetFolder = to_folder.into();
        let new_path = format!("assets/{}", folder.to_path("stats.ron".into()));
        crate::utils::read_ron::<RawStatsList>(new_path)
    }

    pub fn apply_to_character(mut self, ent: Entity, comms: &mut Commands) {
        let mut stats_list = StatList::new();
        for stats_entry in self.0.drain(..) {
            let mut ec = comms.entity(ent);
            stats_entry.kind.to_component(&mut ec, stats_entry.val);
            let stat = Stat {
                base_value: stats_entry.val,
                modifiers: Vec::new(),
                current: Arc::new(Mutex::new(stats_entry.val)),
            };
            stats_list.list.insert(stats_entry.kind, stat);
        }
        comms.entity(ent).insert(stats_list);
    }
}

/*/// Every character with stats has an entry for the base values and the current values
/// of the stat.
/// We may want to set it up later where some Components just respond to changes in the stats (e.g. Health),
/// but generally speaking the stat values should be contained in here
#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone, Default, Reflect)]
pub struct StatsList {
    // The list of base stat values that this entity has
    pub base: HashMap<StatKind, f32>,
    // The accumlation of values that add into the base stats to become the "current" stats
    pub modifiers: HashMap<StatKind, f32>,
    // The relationships that this entity listens for in order to update their modifiers
    // The inner statKind in the hashmap here is the donor entity's statkind marker,
    // so that if the player's health affected projectile size and area of affect (for example),
    // we could put those both in as <Player, <Health, vec![Projsize, AoE]>>
    pub receives_mods_from: HashMap<Entity, HashMap<StatKind, Vec<StatModifier>>>,
}




impl StatsList {
    pub fn get_current(&self, sk: &StatKind) -> Option<f32> {
        let bv = if let Some(b) = self.base.get(sk) {
            b
        } else {
            return None;
        };
        let modif = if let Some(m) = self.modifiers.get(sk) {
            m
        } else {
            return None;
        };
        Some(bv + modif)
    }
}

/// Sets the base stat to a new value, and then passes along a ChangeMessage based on the change to the CURRENT stat.
/// The current stat is the only one that we care about propagating
pub fn set_base_stat_value(
    ent: Entity,
    stats: Mut<StatsList>,
    kind: StatKind,
    to: f32,
) -> Option<CurrentStatChangeMessage> {
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Reflect)]
pub struct StatModifier {
    pub method: StatModifierMethod,
    pub modifies: StatKind,
    pub c_value: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Reflect)]
pub enum StatModifierMethod {
    SetModifierToChangedStatValue,
    ScaleModifierFromChangedValue { scale_factor: f32 },
}

/// Our model of stat changes is that "this" entity listens for changes to the
/// stats that it depends on, so that way we can publish events where stats change, and then react to them
#[derive(Message)]
pub struct CurrentStatChangeMessage {
    pub entity: Entity,
}

*/
/*


#[derive(Debug, Clone, Copy, Deserialize, Serialize, Reflect)]
#[reflect(Default)]
pub enum StatKind {
    AttackRange(AttackRange),
    Armor(Armor),
    CritChance(CritChance),
    CritDamage(CritDamage),
    CDR(CooldownRate),
    Damage(Damage),
    EffDuration(EffectDuration),
    EffSize(EffectSize),
    Evasion(Evasion),
    Health(Health),
    HealthRegen(HPRegen),
    Luck(Luck),
    LifeSteal(LifeSteal),
    MS(MovementSpeed),
    PickupR(PickupRadius),
    ProjCount(ProjectileCount),
    ProjSpeed(ProjectileSpeed),
    Shield(Shield),
    Thorns(Thorns),
    XPGain(XPGain),
}

impl Default for StatKind {
    fn default() -> Self {
        Self::Luck(Luck(1.0))
    }
}

impl StatKind {
    fn to_component(&self, ec: &mut EntityCommands) {
        match self {
            Self::AttackRange(r) => {
                ec.insert(*r);
            }
            Self::Armor(a) => {
                ec.insert(*a);
            }
            Self::CDR(c) => {
                ec.insert(*c);
            }
            Self::CritChance(cc) => {
                ec.insert(*cc);
            }
            Self::CritDamage(cd) => {
                ec.insert(*cd);
            }
            Self::Damage(d) => {
                ec.insert(*d);
            }
            Self::EffDuration(ed) => {
                ec.insert(*ed);
            }
            Self::EffSize(es) => {
                ec.insert(*es);
            }
            Self::Evasion(ev) => {
                ec.insert(*ev);
            }
            Self::Health(hp) => {
                ec.insert(*hp);
            }
            Self::HealthRegen(hr) => {
                ec.insert(*hr);
            }
            Self::Luck(l) => {
                ec.insert(*l);
            }
            Self::LifeSteal(ls) => {
                ec.insert(*ls);
            }
            Self::MS(m) => {
                ec.insert(*m);
            }
            Self::PickupR(pr) => {
                ec.insert(*pr);
            }
            Self::ProjCount(pc) => {
                ec.insert(*pc);
            }
            Self::ProjSpeed(ps) => {
                ec.insert(*ps);
            }
            Self::Shield(s) => {
                ec.insert(*s);
            }
            Self::Thorns(t) => {
                ec.insert(*t);
            }
            Self::XPGain(xp) => {
                ec.insert(*xp);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Reflect, Debug, Clone, Default)]
#[reflect(Default)]
pub struct RawStatsList(Vec<StatKind>);

impl RawStatsList {
    pub fn import_stats(to_folder: impl Into<AssetFolder>) -> Self {
        let folder: AssetFolder = to_folder.into();
        let new_path = format!("assets/{}", folder.to_path("stats.ron".into()));
        crate::utils::read_ron::<RawStatsList>(new_path)
    }

    pub fn apply_to_character(mut self, ent: Entity, comms: &mut Commands) {
        let mut ec = comms.entity(ent);
        for sk in self.0.drain(..) {
            sk.to_component(&mut ec);
        }
    }
}
*/
