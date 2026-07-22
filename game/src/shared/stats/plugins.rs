use super::{apply_stat_modifier, components::*, relationships::StatRelationshipsPlugin, xp::*};
use crate::shared::{
    combat::CombatSystemSet,
    damage::{DamageBuffer, DamageResult, DamageResultMessage},
    states::InGameState,
};
use bevy::prelude::*;
use lightyear::prelude::*;
use std::marker::PhantomData;

pub struct SharedStatsPlugin;
impl Plugin for SharedStatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StatComponentPlugin);
        app.add_plugins(StatRelationshipsPlugin);
        app.add_systems(
            Update,
            apply_stat_modifier.run_if(in_state(InGameState::InGame)),
        )
        .add_systems(
            FixedUpdate,
            (apply_thorns_damage.in_set(CombatSystemSet::PreCombat),)
                .run_if(in_state(InGameState::InGame)),
        );
    }
}

pub struct StatsProtocolPlugin;

impl Plugin for StatsProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.component::<Armor>().predict();
        app.component::<AttackRange>().predict();
        app.component::<CritChance>().predict();
        app.component::<CritDamage>().predict();
        app.component::<CooldownRate>().predict();
        app.component::<Damage>().predict();
        app.component::<EffectSize>().predict();
        app.component::<EffectDuration>().predict();
        app.component::<Health>().predict();
        app.component::<HealthRegen>().predict();
        app.component::<Luck>().predict();
        app.component::<LifeSteal>().predict();
        app.component::<MovementSpeed>().predict();
        app.component::<PickupRadius>().predict();
        app.component::<ProjectileBounces>().predict();
        app.component::<ProjectileCount>().predict();
        app.component::<ProjectileSpeed>().predict();
        app.component::<Shield>().predict();
        app.component::<Thorns>().predict();
        app.component::<XPGain>().predict();
        app.component::<XPManager>().predict();
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
            StatComponentInnerPlugin::<ProjectileBounces>::new(),
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

/// Will want to revisit this, as this currently
///
/// 1. Can proc off of the thorns of another infinitely
/// 2. applies percentage of damage,
/// 3. Can crit
pub fn apply_thorns_damage(
    mut messages: MessageReader<DamageResultMessage>,
    q_thorns_user: Query<&Thorns>,
    mut q_receiver: Query<&mut DamageBuffer>,
) {
    for result in messages.read() {
        if let Ok(thorns) = q_thorns_user.get(result.damaged_entity) {
            if let Ok(mut damage) = q_receiver.get_mut(result.damaging_entity) {
                let dealt = match result.result {
                    DamageResult::Apply(val) => val,
                    _ => 0.0,
                };
                let to_deal = (thorns.0 * 0.01) * dealt;
                damage.push_damage(result.damaged_entity, to_deal)
            }
        }
    }
}
