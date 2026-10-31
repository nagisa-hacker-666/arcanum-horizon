//! The spell economy, in one place.
//!
//! Every number a spell's cost, power, range or cooldown is derived from lives
//! here, so tuning the mod means editing this file and re-running the tests in
//! `tests/balance_tests.rs`.

/// The highest level a mage reaches.
pub const MAX_LEVEL: u32 = 50;

/// Mana at level 1, before any point in Vitality.
pub const BASE_MANA: f32 = 100.0;

/// Mana each level adds.
pub const MANA_PER_LEVEL: f32 = 4.0;

/// Mana regenerated each server tick, before Vitality and Concentration.
pub const MANA_REGEN_PER_TICK: f32 = 0.35;

/// Ticks without casting before resonance starts to decay.
pub const RESONANCE_DECAY_DELAY: u32 = 100;

/// Resonance lost each tick past that delay.
pub const RESONANCE_DECAY_PER_TICK: f32 = 0.08;

/// Resonance a cast generates, per point of spell power.
pub const RESONANCE_PER_POWER: f32 = 0.35;

/// Resonance needed for an ultimate.
pub const RESONANCE_MAX: f32 = 100.0;

/// Damage a cast deals, per point of spell power.
pub const DAMAGE_PER_POWER: f32 = 0.55;

/// Level 10 is where a mage picks a specialization.
pub const SPECIALIZATION_LEVEL: u32 = 10;

/// Talent points a mage gets per level.
pub const STAT_POINTS_PER_LEVEL: u32 = 2;

/// What each point of a stat is worth, as a fraction.
pub const POWER_PER_POINT: f32 = 0.010;
pub const MASTERY_COST_PER_POINT: f32 = 0.005;
pub const PRECISION_COOLDOWN_PER_POINT: f32 = 0.005;
pub const VITALITY_MANA_PER_POINT: f32 = 1.5;
pub const CONCENTRATION_REGEN_PER_POINT: f32 = 0.010;

/// The shortest cooldown any spell can end up with, once Precision and a
/// modifier have taken their share.
pub const MIN_SPELL_COOLDOWN: u32 = 10;

/// The largest bonus elemental mastery can give, at 100 mastery.
pub const MASTERY_MAX_BONUS: f32 = 0.25;

/// The bonus a specialization gives in its own school.
pub const SPECIALIZATION_BONUS: f32 = 0.15;

/// Hard ceilings on a spell the builder will accept.
///
/// These are what stop a modular system from producing one abusive
/// combination: a spell may never cost more than [`Limits::max_cost`] mana,
/// hit harder than [`Limits::max_power`], reach further than
/// [`Limits::max_range`] or lock the caster out for longer than
/// [`Limits::max_cooldown`] ticks. The efficiency floor additionally refuses a
/// spell that is cheaper than [`MIN_COST_PER_POWER`] times its own power.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Limits {
    pub max_cost: f32,
    pub max_power: f32,
    pub max_range: f32,
    pub min_cooldown: u32,
    pub max_cooldown: u32,
    pub max_modifiers: usize,
}

/// A spell must cost at least this much per point of power. Without it,
/// "Efficient" stacked on a cheap shape would beat every other combination.
pub const MIN_COST_PER_POWER: f32 = 0.9;

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_cost: 140.0,
            max_power: 60.0,
            max_range: 32.0,
            min_cooldown: 10,
            max_cooldown: 1200,
            max_modifiers: 3,
        }
    }
}

impl Limits {
    /// A copy with a different ceiling, for a school or a game mode with its
    /// own economy.
    pub fn with_max_cost(mut self, max_cost: f32) -> Self {
        self.max_cost = max_cost;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_positive_and_ordered() {
        let limits = Limits::default();
        assert!(limits.max_cost > 0.0);
        assert!(limits.max_power > 0.0);
        assert!(limits.max_range > 0.0);
        assert!(limits.min_cooldown < limits.max_cooldown);
        assert!(limits.max_modifiers > 0);
    }
}
