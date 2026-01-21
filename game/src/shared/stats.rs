use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

pub mod components;
pub mod editor;
pub mod xp;

use components::*;
use xp::LevelManager;

use crate::utils::AssetFolder;

pub struct StatsProtocolPlugin;

impl Plugin for StatsProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Armor>().add_prediction();
        app.register_component::<AttackRange>().add_prediction();
        app.register_component::<CritChance>().add_prediction();
        app.register_component::<CritDamage>().add_prediction();
        app.register_component::<CooldownRate>().add_prediction();
        app.register_component::<Damage>().add_prediction();
        app.register_component::<EffectSize>().add_prediction();
        app.register_component::<EffectDuration>().add_prediction();
        app.register_component::<Health>().add_prediction();
        app.register_component::<HealthRegen>().add_prediction();
        app.register_component::<Luck>().add_prediction();
        app.register_component::<LifeSteal>().add_prediction();
        app.register_component::<MovementSpeed>().add_prediction();
        app.register_component::<PickupRadius>().add_prediction();
        app.register_component::<ProjectileCount>().add_prediction();
        app.register_component::<ProjectileSpeed>().add_prediction();
        app.register_component::<Shield>().add_prediction();
        app.register_component::<Thorns>().add_prediction();
        app.register_component::<XPGain>().add_prediction();
        app.register_component::<LevelManager>().add_prediction();
    }
}

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
