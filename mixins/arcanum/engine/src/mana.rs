//! Mana and resonance: the two resources a cast touches.

use crate::balance;

/// A caster's mana.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mana {
    current: f32,
    max: f32,
    regen_per_tick: f32,
}

impl Mana {
    /// A full pool of `max` mana that regenerates at the base rate.
    pub fn new(max: f32) -> Self {
        Self {
            current: max.max(0.0),
            max: max.max(0.0),
            regen_per_tick: balance::MANA_REGEN_PER_TICK,
        }
    }

    /// A pool with a given level: what a save restores.
    pub(crate) fn from_parts(current: f32, max: f32) -> Self {
        let max = max.max(0.0);
        Self {
            current: current.clamp(0.0, max),
            max,
            regen_per_tick: balance::MANA_REGEN_PER_TICK,
        }
    }

    pub fn current(&self) -> f32 {
        self.current
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    /// How full the pool is, from 0.0 to 1.0: what the bar draws.
    pub fn ratio(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.current / self.max).clamp(0.0, 1.0)
        }
    }

    /// Sets the ceiling, keeping what is left and never going over.
    pub fn set_max(&mut self, max: f32) {
        self.max = max.max(0.0);
        self.current = self.current.min(self.max);
    }

    /// Sets the regeneration rate, in mana per tick.
    pub fn set_regen(&mut self, per_tick: f32) {
        self.regen_per_tick = per_tick.max(0.0);
    }

    /// Regenerates over `ticks` ticks, never past the ceiling.
    pub fn tick(&mut self, ticks: u32) {
        if ticks == 0 {
            return;
        }
        self.current = (self.current + self.regen_per_tick * ticks as f32).min(self.max);
    }

    /// Spends `cost`, or nothing at all when there is not enough.
    pub fn spend(&mut self, cost: f32) -> bool {
        if cost <= 0.0 {
            return true;
        }
        if self.current + 1e-4 < cost {
            return false;
        }
        self.current = (self.current - cost).max(0.0);
        true
    }

    pub fn restore(&mut self, amount: f32) {
        if amount <= 0.0 {
            return;
        }
        self.current = (self.current + amount).min(self.max);
    }

    /// Empties the pool: what dying does.
    pub fn empty(&mut self) {
        self.current = 0.0;
    }

    pub fn is_empty(&self) -> bool {
        self.current <= 0.0
    }
}

/// Resonance: the resource a mage builds by casting, and spends on an
/// ultimate. It decays when the mage stops casting, so an ultimate is the
/// reward for a sustained fight rather than for waiting.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Resonance {
    value: f32,
    /// Ticks since the last cast.
    idle: u32,
}

impl Resonance {
    pub fn value(&self) -> f32 {
        self.value
    }

    /// How charged it is, from 0.0 to 1.0.
    pub fn ratio(&self) -> f32 {
        (self.value / balance::RESONANCE_MAX).clamp(0.0, 1.0)
    }

    pub fn is_ready(&self) -> bool {
        self.value >= balance::RESONANCE_MAX - 1e-4
    }

    /// Adds what a cast of `power` power generates, and restarts the idle
    /// countdown.
    pub fn gain(&mut self, power: f32) {
        if power <= 0.0 {
            return;
        }
        self.value =
            (self.value + power * balance::RESONANCE_PER_POWER).min(balance::RESONANCE_MAX);
        self.idle = 0;
    }

    /// Runs a tick of decay.
    pub fn tick(&mut self) {
        self.idle = self.idle.saturating_add(1);
        if self.idle > balance::RESONANCE_DECAY_DELAY {
            self.value = (self.value - balance::RESONANCE_DECAY_PER_TICK).max(0.0);
        }
    }

    /// Spends a full charge, or nothing at all.
    pub fn spend_ultimate(&mut self) -> bool {
        if !self.is_ready() {
            return false;
        }
        self.value = 0.0;
        self.idle = 0;
        true
    }

    /// Forgets everything: what dying does.
    pub fn clear(&mut self) {
        self.value = 0.0;
        self.idle = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_pool_is_full() {
        let mana = Mana::new(100.0);
        assert_eq!(mana.current(), 100.0);
        assert_eq!(mana.ratio(), 1.0);
    }

    #[test]
    fn spending_stops_at_zero_and_is_all_or_nothing() {
        let mut mana = Mana::new(30.0);
        assert!(!mana.spend(40.0));
        assert_eq!(mana.current(), 30.0, "a refused cast spends nothing");
        assert!(mana.spend(30.0));
        assert_eq!(mana.current(), 0.0);
        assert!(mana.is_empty());
    }

    #[test]
    fn regeneration_never_overflows() {
        let mut mana = Mana::new(10.0);
        mana.spend(5.0);
        mana.tick(10_000);
        assert_eq!(mana.current(), 10.0);
        assert_eq!(mana.ratio(), 1.0);
    }

    #[test]
    fn a_smaller_ceiling_clamps_what_is_left() {
        let mut mana = Mana::new(100.0);
        mana.set_max(40.0);
        assert_eq!(mana.current(), 40.0);
    }

    #[test]
    fn resonance_charges_then_decays() {
        let mut resonance = Resonance::default();
        for _ in 0..20 {
            resonance.gain(30.0);
        }
        assert!(resonance.is_ready());
        assert!(resonance.spend_ultimate());
        assert_eq!(resonance.value(), 0.0);

        resonance.gain(10.0);
        let charged = resonance.value();
        for _ in 0..balance::RESONANCE_DECAY_DELAY {
            resonance.tick();
        }
        assert_eq!(resonance.value(), charged, "no decay before the delay");
        for _ in 0..100 {
            resonance.tick();
        }
        assert!(resonance.value() < charged);
    }

    #[test]
    fn an_unready_ultimate_spends_nothing() {
        let mut resonance = Resonance::default();
        resonance.gain(5.0);
        assert!(!resonance.spend_ultimate());
        assert!(resonance.value() > 0.0);
    }
}
