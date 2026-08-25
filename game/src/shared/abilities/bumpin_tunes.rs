use super::*;
use bevy::prelude::*;

#[derive(Component, Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BumpTunes;

pub struct BumpTunesPlugin;
impl Plugin for BumpTunesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_bump_tunes_pulse_timer.run_if(in_state(InGameState::InGame)),
        );
    }
}

pub fn bump_tunes<T: Component>() -> impl Scene {
    bsn! {
        #BumpTunes
        BumpTunes
        Ability
        AutoCast
        PulseActivation {timer: Timer::from_seconds(0.25, TimerMode::Repeating)}
        Damage(0.5)
        EffectSize(300.0)
        CooldownRate(0.25)
        Cooldown::new(0.25)
        HasValidators [
            AbilityOffCooldown
        ]
        on(bumpin_tunes_activate::<T>)
    }
}

pub fn bumpin_tunes_activate<T: Component>(
    trigger: On<ActivateAbility>,
    q_holder: Query<&Position, Without<T>>,
    q_ability: Query<(&AbilityOf, &Damage, &EffectSize), With<BumpTunes>>,
    mut q_target: Query<(&Position, &mut HealthBuffer), With<T>>,
) {
    if let Ok((holder, dam, size)) = q_ability.get(trigger.entity) {
        let holder_loc = q_holder.get(holder.0).expect("Player position not found!");
        for (t_pos, mut buff) in &mut q_target {
            if holder_loc.0.distance(t_pos.0) <= size.0 {
                buff.push_damage(trigger.entity, dam.0, None);
            }
        }
    }
}

pub fn render_bump_tunes(
    mut gizmos: Gizmos,
    q_positions: Query<&Position>,
    q_tunes: Query<(&AbilityOf, &EffectSize)>,
) {
    for (holder, size) in &q_tunes {
        let pos = q_positions
            .get(holder.0)
            .expect("holder position not found?");

        gizmos.circle_2d(pos.0, size.0, bevy::color::palettes::basic::BLUE);
    }
}

pub fn update_bump_tunes_pulse_timer(
    mut commands: Commands,
    q_tunes: Query<
        (Entity, &PulseActivation, &CooldownRate),
        (Changed<CooldownRate>, With<BumpTunes>),
    >,
) {
    for (ent, pulses, cdr) in &q_tunes {
        let mut new_timer = PulseActivation {
            timer: Timer::from_seconds(cdr.0, TimerMode::Once),
        };
        new_timer
            .timer
            .tick(Duration::from_secs_f32(cdr.0).mul_f32(pulses.timer.fraction()));
        commands.entity(ent).insert(new_timer);
    }
}
