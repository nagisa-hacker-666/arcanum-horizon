//! The RPG layer: experience, levels, the five stats, elemental mastery,
//! talents and specializations.
//!
//! Everything the mod can act on is derived here. Arcanum does not touch
//! health or movement: the game's own APIs decide those, so Vitality and
//! Concentration work through mana instead (see `docs/BALANCE.md`).

use std::fmt;

use crate::balance::{self, MAX_LEVEL};
use crate::elements::Element;

/// The five stats a mage puts points into.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stat {
    /// More damage and healing.
    Power,
    /// Cheaper spells and faster elemental mastery.
    Mastery,
    /// Shorter cooldowns.
    Precision,
    /// A larger mana pool.
    Vitality,
    /// Faster regeneration and resonance.
    Concentration,
}

impl Stat {
    pub const ALL: [Stat; 5] = [
        Stat::Power,
        Stat::Mastery,
        Stat::Precision,
        Stat::Vitality,
        Stat::Concentration,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Stat::Power => "power",
            Stat::Mastery => "mastery",
            Stat::Precision => "precision",
            Stat::Vitality => "vitality",
            Stat::Concentration => "concentration",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Stat::Power => "Power",
            Stat::Mastery => "Mastery",
            Stat::Precision => "Precision",
            Stat::Vitality => "Vitality",
            Stat::Concentration => "Concentration",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.id() == id)
    }
}

/// How many points a mage put into each stat.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub power: u16,
    pub mastery: u16,
    pub precision: u16,
    pub vitality: u16,
    pub concentration: u16,
}

impl Stats {
    pub fn get(&self, stat: Stat) -> u16 {
        match stat {
            Stat::Power => self.power,
            Stat::Mastery => self.mastery,
            Stat::Precision => self.precision,
            Stat::Vitality => self.vitality,
            Stat::Concentration => self.concentration,
        }
    }

    pub fn add(&mut self, stat: Stat) {
        let slot = match stat {
            Stat::Power => &mut self.power,
            Stat::Mastery => &mut self.mastery,
            Stat::Precision => &mut self.precision,
            Stat::Vitality => &mut self.vitality,
            Stat::Concentration => &mut self.concentration,
        };
        *slot = slot.saturating_add(1);
    }

    pub fn total(&self) -> u16 {
        self.power + self.mastery + self.precision + self.vitality + self.concentration
    }

    /// What Power multiplies a spell's power by.
    pub fn power_multiplier(&self) -> f32 {
        1.0 + f32::from(self.power) * balance::POWER_PER_POINT
    }

    /// What Mastery multiplies a spell's cost by. Never below 60%: cost
    /// reduction compounds with the efficiency floor, so it is capped.
    pub fn cost_multiplier(&self) -> f32 {
        (1.0 - f32::from(self.mastery) * balance::MASTERY_COST_PER_POINT).max(0.6)
    }

    /// What Precision multiplies a cooldown by. Never below 65%.
    pub fn cooldown_multiplier(&self) -> f32 {
        (1.0 - f32::from(self.precision) * balance::PRECISION_COOLDOWN_PER_POINT).max(0.65)
    }

    /// Mana Vitality adds to the pool.
    pub fn mana_bonus(&self) -> f32 {
        f32::from(self.vitality) * balance::VITALITY_MANA_PER_POINT
    }

    /// What Concentration multiplies regeneration by.
    pub fn regen_multiplier(&self) -> f32 {
        1.0 + f32::from(self.concentration) * balance::CONCENTRATION_REGEN_PER_POINT
    }
}

/// A talent: a passive a mage unlocks at a level, in a school.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Talent {
    pub id: &'static str,
    pub name: &'static str,
    pub element: Element,
    /// The mage level it needs.
    pub level: u32,
    /// Whether it also needs that school's specialization.
    pub specialization: bool,
    /// What it multiplies the school's power by, on top of 1.0.
    pub power_bonus: f32,
    /// What it multiplies the school's cost by, on top of 1.0.
    pub cost_bonus: f32,
}

/// Two talents per school, five levels apart, so every school has a shape of
/// its own from level 5 on.
pub const TALENTS: [Talent; 12] = [
    Talent {
        id: "pyro_ember",
        name: "Ember Heart",
        element: Element::Pyro,
        level: 5,
        specialization: false,
        power_bonus: 0.06,
        cost_bonus: 1.0,
    },
    Talent {
        id: "pyro_inferno",
        name: "Inferno",
        element: Element::Pyro,
        level: 25,
        specialization: true,
        power_bonus: 0.18,
        cost_bonus: 1.0,
    },
    Talent {
        id: "cryo_calm",
        name: "Cold Calm",
        element: Element::Cryo,
        level: 5,
        specialization: false,
        power_bonus: 0.0,
        cost_bonus: 0.90,
    },
    Talent {
        id: "cryo_glacier",
        name: "Glacier",
        element: Element::Cryo,
        level: 25,
        specialization: true,
        power_bonus: 0.16,
        cost_bonus: 1.0,
    },
    Talent {
        id: "electro_static",
        name: "Static Field",
        element: Element::Electro,
        level: 5,
        specialization: false,
        power_bonus: 0.07,
        cost_bonus: 1.0,
    },
    Talent {
        id: "electro_tempest",
        name: "Tempest",
        element: Element::Electro,
        level: 25,
        specialization: true,
        power_bonus: 0.20,
        cost_bonus: 1.0,
    },
    Talent {
        id: "nature_bloom",
        name: "Bloom",
        element: Element::Nature,
        level: 5,
        specialization: false,
        power_bonus: 0.0,
        cost_bonus: 0.88,
    },
    Talent {
        id: "nature_grove",
        name: "Ancient Grove",
        element: Element::Nature,
        level: 25,
        specialization: true,
        power_bonus: 0.14,
        cost_bonus: 0.95,
    },
    Talent {
        id: "shadow_veil",
        name: "Veil",
        element: Element::Shadow,
        level: 5,
        specialization: false,
        power_bonus: 0.06,
        cost_bonus: 0.95,
    },
    Talent {
        id: "shadow_abyss",
        name: "Abyss",
        element: Element::Shadow,
        level: 25,
        specialization: true,
        power_bonus: 0.22,
        cost_bonus: 1.0,
    },
    Talent {
        id: "arcane_focus",
        name: "Focus",
        element: Element::Arcane,
        level: 5,
        specialization: false,
        power_bonus: 0.05,
        cost_bonus: 0.95,
    },
    Talent {
        id: "arcane_singular",
        name: "Singular Focus",
        element: Element::Arcane,
        level: 25,
        specialization: true,
        power_bonus: 0.20,
        cost_bonus: 1.05,
    },
];

/// The talent with `id`, wherever it lives.
pub fn talent(id: &str) -> Option<&'static Talent> {
    TALENTS.iter().find(|t| t.id == id)
}

/// Why a progression change was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProgressionError {
    LevelTooLow { required: u32, level: u32 },
    NotSpecialized,
    UnknownTalent,
    AlreadyUnlocked,
    NoPoints,
    AlreadySpecialized(Element),
}

impl fmt::Display for ProgressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LevelTooLow { required, level } => {
                write!(f, "needs level {required}, this mage is {level}")
            }
            Self::NotSpecialized => f.write_str("needs a specialization in that school"),
            Self::UnknownTalent => f.write_str("no such talent"),
            Self::AlreadyUnlocked => f.write_str("that talent is already unlocked"),
            Self::NoPoints => f.write_str("no stat points left"),
            Self::AlreadySpecialized(element) => {
                write!(f, "already specialized in {}", element.name())
            }
        }
    }
}

impl std::error::Error for ProgressionError {}

/// A mage's progression: what they have earned and what they have spent.
#[derive(Clone, Debug, PartialEq)]
pub struct Progression {
    /// Experience earned, lifetime.
    pub xp: u64,
    pub level: u32,
    /// Stat points not yet spent.
    pub points: u32,
    pub stats: Stats,
    /// Elemental mastery, 0 to 100, indexed by [`Element::index`].
    mastery: [f32; 6],
    pub specialization: Option<Element>,
    talents: Vec<&'static str>,
}

impl Default for Progression {
    fn default() -> Self {
        Self {
            xp: 0,
            level: 1,
            points: 0,
            stats: Stats::default(),
            mastery: [0.0; 6],
            specialization: None,
            talents: Vec::new(),
        }
    }
}

impl Progression {
    pub fn new() -> Self {
        Self::default()
    }

    /// Experience the level after `level` needs.
    pub fn xp_for_next(level: u32) -> u64 {
        if level >= MAX_LEVEL {
            return 0;
        }
        let l = u64::from(level);
        100 + 55 * l + 8 * l * l
    }

    /// Total experience a mage of `level` has earned.
    pub fn total_xp_for(level: u32) -> u64 {
        let capped = level.min(MAX_LEVEL);
        (1..capped).map(Self::xp_for_next).sum()
    }

    /// The level `xp` experience is worth.
    pub fn level_from_xp(xp: u64) -> u32 {
        let mut level = 1;
        while level < MAX_LEVEL && xp >= Self::total_xp_for(level + 1) {
            level += 1;
        }
        level
    }

    /// Experience into the current level, and what the level needs: what the
    /// HUD's bar draws.
    pub fn level_progress(&self) -> (u64, u64) {
        let floor = Self::total_xp_for(self.level);
        let needed = Self::xp_for_next(self.level);
        (self.xp.saturating_sub(floor), needed)
    }

    /// Adds experience and levels up as far as it goes. Returns the levels
    /// gained, which is what the HUD announces.
    pub fn add_xp(&mut self, xp: u64) -> u32 {
        if xp == 0 || self.level >= MAX_LEVEL {
            return 0;
        }
        self.xp = self.xp.saturating_add(xp);
        let target = Self::level_from_xp(self.xp);
        let gained = target.saturating_sub(self.level);
        if gained > 0 {
            self.level = target;
            self.points = self
                .points
                .saturating_add(gained * balance::STAT_POINTS_PER_LEVEL);
        }
        gained
    }

    /// Spends one unspent point on `stat`.
    pub fn spend_point(&mut self, stat: Stat) -> Result<(), ProgressionError> {
        if self.points == 0 {
            return Err(ProgressionError::NoPoints);
        }
        self.points -= 1;
        self.stats.add(stat);
        Ok(())
    }

    /// The mage's mastery of a school, 0 to 100.
    pub fn mastery(&self, element: Element) -> f32 {
        self.mastery[element.index()]
    }

    /// Raises mastery, never past 100.
    pub fn add_mastery(&mut self, element: Element, amount: f32) {
        if amount <= 0.0 {
            return;
        }
        let slot = &mut self.mastery[element.index()];
        *slot = (*slot + amount * self.stats.regen_multiplier()).min(100.0);
    }

    /// What mastery multiplies that school's power by.
    pub fn mastery_bonus(&self, element: Element) -> f32 {
        self.mastery(element) / 100.0 * balance::MASTERY_MAX_BONUS
    }

    /// Picks a specialization once, from level 10 on.
    pub fn choose_specialization(&mut self, element: Element) -> Result<(), ProgressionError> {
        if self.level < balance::SPECIALIZATION_LEVEL {
            return Err(ProgressionError::LevelTooLow {
                required: balance::SPECIALIZATION_LEVEL,
                level: self.level,
            });
        }
        if let Some(current) = self.specialization {
            return Err(ProgressionError::AlreadySpecialized(current));
        }
        self.specialization = Some(element);
        Ok(())
    }

    /// Unlocks a talent when its level and specialization are met.
    pub fn unlock_talent(&mut self, id: &str) -> Result<(), ProgressionError> {
        let talent = talent(id).ok_or(ProgressionError::UnknownTalent)?;
        if self.talents.contains(&talent.id) {
            return Err(ProgressionError::AlreadyUnlocked);
        }
        if self.level < talent.level {
            return Err(ProgressionError::LevelTooLow {
                required: talent.level,
                level: self.level,
            });
        }
        if talent.specialization && self.specialization != Some(talent.element) {
            return Err(ProgressionError::NotSpecialized);
        }
        self.talents.push(talent.id);
        Ok(())
    }

    pub fn talents(&self) -> &[&'static str] {
        &self.talents
    }

    /// Puts a whole progression back after loading a save. Talents are
    /// restored one by one, after this, by [`Self::restore_talent`].
    pub(crate) fn restore(
        &mut self,
        xp: u64,
        level: u32,
        points: u32,
        stats: Stats,
        mastery: [f32; 6],
        specialization: Option<Element>,
    ) {
        self.xp = xp;
        self.level = level.clamp(1, MAX_LEVEL);
        self.points = points;
        self.stats = stats;
        self.mastery = mastery;
        self.specialization = specialization;
        self.talents.clear();
    }

    /// Puts a talent back after loading a save, without re-checking levels:
    /// the save was written when they were met.
    pub(crate) fn restore_talent(&mut self, id: &'static str) {
        if !self.talents.contains(&id) {
            self.talents.push(id);
        }
    }

    /// What Power, mastery, talents and the specialization multiply a
    /// school's spell power by.
    pub fn power_multiplier(&self, element: Element) -> f32 {
        let talent_bonus: f32 = self
            .talents
            .iter()
            .filter_map(|id| talent(id))
            .filter(|t| t.element == element)
            .map(|t| t.power_bonus)
            .sum();
        let specialization = if self.specialization == Some(element) {
            balance::SPECIALIZATION_BONUS
        } else {
            0.0
        };
        self.stats.power_multiplier() + self.mastery_bonus(element) + talent_bonus + specialization
    }

    /// What Mastery and talents multiply a school's spell cost by.
    pub fn cost_multiplier(&self, element: Element) -> f32 {
        let talent_scale: f32 = self
            .talents
            .iter()
            .filter_map(|id| talent(id))
            .filter(|t| t.element == element)
            .map(|t| t.cost_bonus)
            .product();
        (self.stats.cost_multiplier() * talent_scale).clamp(0.35, 1.5)
    }

    /// What Precision multiplies every cooldown by.
    pub fn cooldown_multiplier(&self) -> f32 {
        self.stats.cooldown_multiplier()
    }

    /// Mana the level and Vitality add to the base pool.
    pub fn max_mana(&self) -> f32 {
        balance::BASE_MANA
            + (self.level.saturating_sub(1) as f32) * balance::MANA_PER_LEVEL
            + self.stats.mana_bonus()
    }

    /// What Concentration multiplies regeneration by.
    pub fn regen_multiplier(&self) -> f32 {
        self.stats.regen_multiplier()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_curve_is_increasing() {
        for level in 1..MAX_LEVEL - 1 {
            assert!(
                Progression::xp_for_next(level) < Progression::xp_for_next(level + 1),
                "level {level}"
            );
        }
        assert_eq!(Progression::xp_for_next(MAX_LEVEL), 0);
    }

    #[test]
    fn xp_maps_to_levels_and_stops_at_the_cap() {
        assert_eq!(Progression::level_from_xp(0), 1);
        assert_eq!(Progression::level_from_xp(Progression::total_xp_for(2)), 2);
        assert_eq!(
            Progression::level_from_xp(Progression::total_xp_for(MAX_LEVEL) * 100),
            MAX_LEVEL
        );
    }

    #[test]
    fn levelling_hands_out_points() {
        let mut progression = Progression::new();
        let gained = progression.add_xp(Progression::xp_for_next(1));
        assert_eq!(gained, 1);
        assert_eq!(progression.level, 2);
        assert_eq!(progression.points, balance::STAT_POINTS_PER_LEVEL);
        let (into, needed) = progression.level_progress();
        assert!(into < needed);
    }

    #[test]
    fn a_point_can_only_be_spent_when_there_is_one() {
        let mut progression = Progression::new();
        assert_eq!(
            progression.spend_point(Stat::Power),
            Err(ProgressionError::NoPoints)
        );
        progression.add_xp(Progression::xp_for_next(1));
        assert!(progression.spend_point(Stat::Power).is_ok());
        assert_eq!(progression.stats.power, 1);
    }

    #[test]
    fn specialization_waits_for_level_ten_and_is_picked_once() {
        let mut progression = Progression::new();
        assert!(matches!(
            progression.choose_specialization(Element::Pyro),
            Err(ProgressionError::LevelTooLow { .. })
        ));
        progression.add_xp(Progression::total_xp_for(10));
        assert_eq!(progression.level, 10);
        assert!(progression.choose_specialization(Element::Pyro).is_ok());
        assert!(matches!(
            progression.choose_specialization(Element::Cryo),
            Err(ProgressionError::AlreadySpecialized(Element::Pyro))
        ));
    }

    #[test]
    fn talents_gate_on_level_and_specialization() {
        let mut progression = Progression::new();
        assert_eq!(
            progression.unlock_talent("pyro_ember"),
            Err(ProgressionError::LevelTooLow {
                required: 5,
                level: 1
            })
        );
        progression.add_xp(Progression::total_xp_for(25));
        assert!(progression.unlock_talent("pyro_ember").is_ok());
        assert_eq!(
            progression.unlock_talent("pyro_inferno"),
            Err(ProgressionError::NotSpecialized)
        );
        progression
            .choose_specialization(Element::Pyro)
            .expect("level 25");
        assert!(progression.unlock_talent("pyro_inferno").is_ok());
        assert_eq!(
            progression.unlock_talent("pyro_inferno"),
            Err(ProgressionError::AlreadyUnlocked)
        );
        assert_eq!(
            progression.unlock_talent("nope"),
            Err(ProgressionError::UnknownTalent)
        );
    }

    #[test]
    fn multipliers_grow_with_investment_but_stay_bounded() {
        let mut progression = Progression::new();
        for _ in 0..500 {
            for stat in Stat::ALL {
                progression.stats.add(stat);
            }
        }
        assert!(progression.stats.cost_multiplier() >= 0.6);
        assert!(progression.stats.cooldown_multiplier() >= 0.65);
        assert!(progression.power_multiplier(Element::Pyro) > 1.0);
    }

    #[test]
    fn mastery_saturates() {
        let mut progression = Progression::new();
        for _ in 0..100 {
            progression.add_mastery(Element::Cryo, 10.0);
        }
        assert_eq!(progression.mastery(Element::Cryo), 100.0);
        assert_eq!(
            progression.mastery_bonus(Element::Cryo),
            balance::MASTERY_MAX_BONUS
        );
    }

    #[test]
    fn every_talent_names_a_known_school_and_id() {
        for talent in TALENTS {
            assert!(talent.power_bonus >= 0.0);
            assert!(talent.cost_bonus > 0.0);
            assert!(talent.level >= 5);
            assert!(Element::from_id(talent.element.id()).is_some());
        }
    }
}
