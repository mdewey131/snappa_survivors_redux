use std::marker::PhantomData;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub mod components;
pub mod editor;

use components::*;

use crate::utils::AssetFolder;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Reflect)]
#[reflect(Default)]
pub enum StatKind {
    AttackRange(AttackRange),
    CritChance(CritChance),
    CritDamage(CritDamage),
    CDR(CooldownRate),
    Damage(Damage),
    EffDuration(EffectDuration),
    EffSize(EffectSize),
    Health(Health),
    Luck(Luck),
    MS(MovementSpeed),
    PickupR(PickupRadius),
    ProjCount(ProjectileCount),
    ProjSpeed(ProjectileSpeed),
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
            Self::Health(hp) => {
                ec.insert(*hp);
            }
            Self::Luck(l) => {
                ec.insert(*l);
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
