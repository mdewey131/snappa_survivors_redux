use super::{components::*, xp::*};
use crate::shared::{combat::CombatSystemSet, states::InGameState};
use bevy::prelude::*;
use lightyear::prelude::*;
use std::marker::PhantomData;

pub struct SharedStatsPlugin;
impl Plugin for SharedStatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StatComponentPlugin);
    }
}

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

pub struct StatComponentPlugin;
impl Plugin for StatComponentPlugin {
    fn build(&self, app: &mut App) {
        // Split these up otherwise it doesn't impl Plugins<>
        app.add_plugins((
            StatComponentInnerPlugin::<AttackRange>::new(),
            StatComponentInnerPlugin::<Armor>::new(),
            StatComponentInnerPlugin::<CritChance>::new(),
            StatComponentInnerPlugin::<CritDamage>::new(),
            StatComponentInnerPlugin::<CooldownRate>::new(),
            StatComponentInnerPlugin::<Damage>::new(),
            StatComponentInnerPlugin::<EffectDuration>::new(),
            StatComponentInnerPlugin::<EffectSize>::new(),
            StatComponentInnerPlugin::<Evasion>::new(),
            StatComponentInnerPlugin::<Health>::new(),
            StatComponentInnerPlugin::<HealthRegen>::new(),
            StatComponentInnerPlugin::<Luck>::new(),
            StatComponentInnerPlugin::<LifeSteal>::new(),
        ));
        app.add_plugins((
            StatComponentInnerPlugin::<MovementSpeed>::new(),
            StatComponentInnerPlugin::<PickupRadius>::new(),
            StatComponentInnerPlugin::<ProjectileCount>::new(),
            StatComponentInnerPlugin::<ProjectileSpeed>::new(),
            StatComponentInnerPlugin::<Shield>::new(),
            StatComponentInnerPlugin::<Thorns>::new(),
            StatComponentInnerPlugin::<XPGain>::new(),
        ));
    }
}

pub struct StatComponentInnerPlugin<SC> {
    _mark: PhantomData<SC>,
}
impl<SC: StatComponent> StatComponentInnerPlugin<SC> {
    fn new() -> Self {
        Self { _mark: PhantomData }
    }
}
impl<SC: StatComponent> Plugin for StatComponentInnerPlugin<SC> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPostUpdate,
            (SC::update_stat_component)
                .in_set(CombatSystemSet::Last)
                .run_if(in_state(InGameState::InGame)),
        );
    }
}
