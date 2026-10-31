//! Per-spell cooldowns, counted in server ticks.

use std::collections::BTreeMap;

/// The cooldowns a caster is waiting on.
///
/// A `BTreeMap` rather than a `HashMap` so a save, a log and a test always
/// see the same order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cooldowns {
    remaining: BTreeMap<String, u32>,
}

impl Cooldowns {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts (or restarts) a spell's cooldown.
    pub fn put(&mut self, spell_id: &str, ticks: u32) {
        if ticks == 0 {
            self.remaining.remove(spell_id);
        } else {
            self.remaining.insert(spell_id.to_owned(), ticks);
        }
    }

    /// Ticks left before `spell_id` may be cast; 0 when it is ready.
    pub fn remaining(&self, spell_id: &str) -> u32 {
        self.remaining.get(spell_id).copied().unwrap_or(0)
    }

    pub fn ready(&self, spell_id: &str) -> bool {
        self.remaining(spell_id) == 0
    }

    /// Runs `ticks` ticks, forgetting the spells that came back.
    pub fn tick(&mut self, ticks: u32) {
        if ticks == 0 || self.remaining.is_empty() {
            return;
        }
        self.remaining.retain(|_, left| {
            *left = left.saturating_sub(ticks);
            *left > 0
        });
    }

    /// Forgets every cooldown.
    pub fn clear(&mut self) {
        self.remaining.clear();
    }

    /// The spells still cooling down, in id order.
    pub fn active(&self) -> impl Iterator<Item = (&str, u32)> {
        self.remaining
            .iter()
            .map(|(id, ticks)| (id.as_str(), *ticks))
    }

    pub fn is_empty(&self) -> bool {
        self.remaining.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cooldown_counts_down() {
        let mut cooldowns = Cooldowns::new();
        cooldowns.put("pyro_fireball", 20);
        assert!(!cooldowns.ready("pyro_fireball"));
        cooldowns.tick(19);
        assert_eq!(cooldowns.remaining("pyro_fireball"), 1);
        cooldowns.tick(1);
        assert!(cooldowns.ready("pyro_fireball"));
        assert!(cooldowns.is_empty(), "a finished cooldown is forgotten");
    }

    #[test]
    fn an_unknown_spell_is_ready() {
        assert!(Cooldowns::new().ready("nothing"));
    }

    #[test]
    fn zero_ticks_means_ready() {
        let mut cooldowns = Cooldowns::new();
        cooldowns.put("a", 10);
        cooldowns.put("a", 0);
        assert!(cooldowns.ready("a"));
    }

    #[test]
    fn active_lists_in_id_order() {
        let mut cooldowns = Cooldowns::new();
        cooldowns.put("b", 5);
        cooldowns.put("a", 7);
        let ids: Vec<&str> = cooldowns.active().map(|(id, _)| id).collect();
        assert_eq!(ids, ["a", "b"]);
    }
}
