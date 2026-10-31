//! A mage: what the server keeps per player, and the only place a cast is
//! allowed to start.
//!
//! The server owns every number here. A client sends "cast" and nothing else;
//! whether the spell exists, whether the mage learned it, whether mana is left
//! and whether the cooldown is over are all decided in [`PlayerState::cast`].

use std::fmt;

use crate::balance;
use crate::cast::{resolve, CastError, EffectPlan};
use crate::cooldown::Cooldowns;
use crate::effects::RenderQuality;
use crate::mana::{Mana, Resonance};
use crate::progression::Progression;
use crate::registry::SpellRegistry;
use crate::spell::{round1, Spell};

/// Why a spell could not be learned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LearnError {
    LevelTooLow { required: u32, level: u32 },
    AlreadyKnown,
}

impl fmt::Display for LearnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LevelTooLow { required, level } => {
                write!(f, "needs level {required}, you are {level}")
            }
            Self::AlreadyKnown => f.write_str("that spell is already in your grimoire"),
        }
    }
}

impl std::error::Error for LearnError {}

/// Everything the server remembers about one mage.
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerState {
    /// The player's UUID, as Horizon reports it (`u128`).
    pub uuid: u128,
    /// The player's name, used in the commands a cast runs.
    pub name: String,
    pub mana: Mana,
    pub resonance: Resonance,
    pub progression: Progression,
    pub cooldowns: Cooldowns,
    known: Vec<String>,
    selected: usize,
}

impl PlayerState {
    /// A level 1 mage who knows nothing yet.
    pub fn new(uuid: u128, name: impl Into<String>) -> Self {
        let progression = Progression::new();
        let mut mana = Mana::new(progression.max_mana());
        mana.set_regen(balance::MANA_REGEN_PER_TICK * progression.regen_multiplier());
        Self {
            uuid,
            name: name.into(),
            mana,
            resonance: Resonance::default(),
            progression,
            cooldowns: Cooldowns::new(),
            known: Vec::new(),
            selected: 0,
        }
    }

    /// Recomputes the mana pool after a level or a stat changed.
    pub fn refresh(&mut self) {
        self.mana.set_max(self.progression.max_mana());
        self.mana
            .set_regen(balance::MANA_REGEN_PER_TICK * self.progression.regen_multiplier());
    }

    /// Runs the server's ticks for this mage.
    pub fn tick(&mut self, ticks: u32) {
        if ticks == 0 {
            return;
        }
        self.mana.tick(ticks);
        self.cooldowns.tick(ticks);
        for _ in 0..ticks {
            self.resonance.tick();
        }
    }

    /// What the spell costs this mage.
    pub fn effective_cost(&self, spell: &Spell) -> f32 {
        round1((spell.cost * self.progression.cost_multiplier(spell.element)).max(1.0))
    }

    /// How long the spell locks this mage out.
    pub fn effective_cooldown(&self, spell: &Spell) -> u32 {
        let scaled = spell.cooldown as f32 * self.progression.cooldown_multiplier();
        (scaled.round() as u32).max(balance::MIN_SPELL_COOLDOWN)
    }

    /// What the spell hits for in this mage's hands.
    pub fn effective_power(&self, spell: &Spell) -> f32 {
        round1(spell.power * self.progression.power_multiplier(spell.element))
    }

    /// Whether the mage knows a spell.
    pub fn knows(&self, id: &str) -> bool {
        self.known.iter().any(|known| known == id)
    }

    /// The spells in the grimoire, in the order they were learned.
    pub fn known(&self) -> &[String] {
        &self.known
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Puts a spell in the grimoire.
    pub fn learn(&mut self, spell: &Spell) -> Result<(), LearnError> {
        if self.knows(&spell.id) {
            return Err(LearnError::AlreadyKnown);
        }
        if self.progression.level < spell.required_level {
            return Err(LearnError::LevelTooLow {
                required: spell.required_level,
                level: self.progression.level,
            });
        }
        self.known.push(spell.id.clone());
        Ok(())
    }

    /// Selects the spell at `index` in the grimoire.
    pub fn select(&mut self, index: usize) -> Option<&str> {
        if index >= self.known.len() {
            return None;
        }
        self.selected = index;
        self.known.get(index).map(String::as_str)
    }

    /// Moves the selection by `delta`, wrapping: what the mouse wheel does.
    pub fn cycle(&mut self, delta: i32) -> Option<&str> {
        if self.known.is_empty() || delta == 0 {
            return self.known.get(self.selected).map(String::as_str);
        }
        let len = self.known.len() as i32;
        self.selected = (self.selected as i32 + delta).rem_euclid(len) as usize;
        self.known.get(self.selected).map(String::as_str)
    }

    /// The spell the wheel has selected, looked up in the server's registry.
    pub fn selected_spell<'a>(&self, registry: &'a SpellRegistry) -> Option<&'a Spell> {
        let id = self.known.get(self.selected)?;
        registry.get(id)
    }

    /// Casts whatever the wheel has selected.
    pub fn try_cast(
        &mut self,
        registry: &SpellRegistry,
        quality: RenderQuality,
    ) -> Result<EffectPlan, CastError> {
        let Some(id) = self.known.get(self.selected).cloned() else {
            return Err(CastError::EmptySelection);
        };
        let spell = registry
            .get(&id)
            .ok_or_else(|| CastError::UnknownSpell(id.clone()))?;
        self.cast(spell, quality)
    }

    /// Validates a cast and, when it is legal, spends the mana, starts the
    /// cooldown, charges resonance and hands the server its commands.
    pub fn cast(&mut self, spell: &Spell, quality: RenderQuality) -> Result<EffectPlan, CastError> {
        if !self.knows(&spell.id) {
            return Err(CastError::NotLearned(spell.id.clone()));
        }
        let remaining = self.cooldowns.remaining(&spell.id);
        if remaining > 0 {
            return Err(CastError::OnCooldown { ticks: remaining });
        }
        let cost = self.effective_cost(spell);
        let available = self.mana.current();
        if available + 1e-4 < cost {
            return Err(CastError::NotEnoughMana {
                needed: cost,
                available,
            });
        }
        let power = self.effective_power(spell);
        let plan = resolve(spell, &self.name, power, quality)?;
        // Everything below only runs once the plan is known to be executable,
        // so a refused cast never costs a mage anything.
        self.mana.spend(cost);
        self.cooldowns
            .put(&spell.id, self.effective_cooldown(spell));
        self.resonance.gain(power);
        self.progression.add_mastery(spell.element, 0.5);
        if self.progression.add_xp(spell.xp_reward()) > 0 {
            self.refresh();
        }
        Ok(plan)
    }

    /// Puts a grimoire back after loading a save.
    pub(crate) fn restore_known(&mut self, known: Vec<String>, selected: usize) {
        self.selected = if known.is_empty() {
            0
        } else {
            selected.min(known.len() - 1)
        };
        self.known = known;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::Element;

    fn mage() -> (PlayerState, SpellRegistry) {
        let registry = SpellRegistry::builtin().expect("balanced");
        let mut player = PlayerState::new(1, "Steve");
        // Learn the whole book, at level 50, so the tests are about casting.
        player
            .progression
            .add_xp(Progression::total_xp_for(balance::MAX_LEVEL));
        player.refresh();
        for spell in registry.all() {
            player.learn(spell).expect("learned");
        }
        (player, registry)
    }

    #[test]
    fn a_cast_spends_mana_and_starts_a_cooldown() {
        let (mut player, registry) = mage();
        let spell = registry.get("pyro_fireball").expect("known spell").clone();
        let mana = player.mana.current();
        let plan = player.cast(&spell, RenderQuality::Low).expect("cast");
        assert!(!plan.commands.is_empty());
        assert!(player.mana.current() < mana);
        assert!(player.cooldowns.remaining(&spell.id) > 0);
        assert!(player.resonance.value() > 0.0);
    }

    #[test]
    fn a_second_cast_waits_for_the_cooldown() {
        let (mut player, registry) = mage();
        let spell = registry.get("pyro_fireball").expect("known spell").clone();
        player.cast(&spell, RenderQuality::Low).expect("first cast");
        match player.cast(&spell, RenderQuality::Low) {
            Err(CastError::OnCooldown { ticks }) => assert!(ticks > 0),
            other => panic!("expected a cooldown, got {other:?}"),
        }
    }

    #[test]
    fn an_unlearned_spell_is_refused_and_costs_nothing() {
        let registry = SpellRegistry::builtin().expect("balanced");
        let mut player = PlayerState::new(2, "Alex");
        let spell = registry.get("pyro_fireball").expect("known spell").clone();
        let mana = player.mana.current();
        assert_eq!(
            player.cast(&spell, RenderQuality::Low),
            Err(CastError::NotLearned(spell.id.clone()))
        );
        assert_eq!(player.mana.current(), mana);
    }

    #[test]
    fn casting_without_mana_is_refused_and_spends_nothing() {
        let (mut player, registry) = mage();
        let spell = registry
            .get("arcane_singularity")
            .expect("known spell")
            .clone();
        player.mana.empty();
        match player.cast(&spell, RenderQuality::Low) {
            Err(CastError::NotEnoughMana { needed, available }) => {
                assert!(needed > 0.0);
                assert_eq!(available, 0.0);
            }
            other => panic!("expected a mana refusal, got {other:?}"),
        }
        assert!(player.mana.is_empty());
        assert!(
            player.cooldowns.ready(&spell.id),
            "a refused cast starts no cooldown"
        );
    }

    #[test]
    fn a_caster_with_no_name_is_refused_and_keeps_their_mana() {
        let (mut player, registry) = mage();
        player.name = "not a name".to_owned();
        let spell = registry.get("pyro_fireball").expect("known spell").clone();
        let mana = player.mana.current();
        assert_eq!(
            player.cast(&spell, RenderQuality::Low),
            Err(CastError::InvalidOrigin)
        );
        assert_eq!(player.mana.current(), mana);
    }

    #[test]
    fn level_gates_a_spell() {
        let registry = SpellRegistry::builtin().expect("balanced");
        let mut player = PlayerState::new(3, "Novice");
        let spell = registry
            .get("arcane_singularity")
            .expect("known spell")
            .clone();
        assert!(matches!(
            player.learn(&spell),
            Err(LearnError::LevelTooLow { required: 40, .. })
        ));
        let first = registry.get("pyro_fireball").expect("known spell");
        player.learn(first).expect("level 1 spell");
        assert_eq!(player.learn(first), Err(LearnError::AlreadyKnown));
    }

    #[test]
    fn the_wheel_wraps_both_ways() {
        let (mut player, _) = mage();
        assert_eq!(player.selected_index(), 0);
        let last = player.known().last().cloned();
        assert_eq!(player.cycle(-1).map(str::to_owned), last);
        let first = player.known().first().cloned();
        assert_eq!(player.cycle(1).map(str::to_owned), first);
        assert_eq!(player.select(99), None);
    }

    #[test]
    fn the_effective_numbers_follow_the_progression() {
        let (player, registry) = mage();
        let spell = registry.get("pyro_fireball").expect("known spell");
        assert!(player.effective_power(spell) >= spell.power);
        assert!(player.effective_cost(spell) <= spell.cost);
        assert!(player.effective_cooldown(spell) >= balance::MIN_SPELL_COOLDOWN);
    }

    #[test]
    fn every_element_has_a_spell_to_cast() {
        let (mut player, registry) = mage();
        for element in Element::ALL {
            let spell = registry
                .by_school(element)
                .next()
                .expect("each school has spells")
                .clone();
            player.cooldowns.clear();
            player.mana.restore(player.mana.max());
            assert!(
                player.cast(&spell, RenderQuality::Low).is_ok(),
                "{} could not be cast",
                spell.id
            );
        }
    }
}
