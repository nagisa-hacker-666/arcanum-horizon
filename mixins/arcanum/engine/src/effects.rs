//! What a spell does when it lands, and how much visual effort the client
//! spends on it.

use std::fmt;

/// What a spell does to what it reaches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Deals damage as the school's damage type.
    Damage,
    /// Restores health.
    Heal,
    /// Slows what it reaches.
    Slow,
    /// Holds what it reaches in place.
    Root,
    /// Wraps the caster in absorption and resistance.
    Shield,
    /// Deals damage and heals the caster for a share of it.
    Drain,
    /// Moves the caster.
    Teleport,
}

impl Effect {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Damage => "damage",
            Self::Heal => "heal",
            Self::Slow => "slow",
            Self::Root => "root",
            Self::Shield => "shield",
            Self::Drain => "drain",
            Self::Teleport => "teleport",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Damage => "Damage",
            Self::Heal => "Healing",
            Self::Slow => "Slowing",
            Self::Root => "Rooting",
            Self::Shield => "Shielding",
            Self::Drain => "Draining",
            Self::Teleport => "Teleportation",
        }
    }

    pub const ALL: [Effect; 7] = [
        Self::Damage,
        Self::Heal,
        Self::Slow,
        Self::Root,
        Self::Shield,
        Self::Drain,
        Self::Teleport,
    ];

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.id() == id)
    }

    /// Mana the effect adds before the school's multiplier.
    pub const fn base_cost(self) -> f32 {
        match self {
            Self::Damage => 4.0,
            Self::Heal => 3.0,
            Self::Slow => 3.0,
            Self::Root => 6.0,
            Self::Shield => 5.0,
            Self::Drain => 5.0,
            Self::Teleport => 4.0,
        }
    }

    /// Power the effect adds before the school's multiplier.
    pub const fn base_power(self) -> f32 {
        match self {
            Self::Damage => 6.0,
            Self::Heal => 4.0,
            Self::Slow => 3.0,
            Self::Root => 5.0,
            Self::Shield => 5.0,
            Self::Drain => 5.0,
            Self::Teleport => 0.0,
        }
    }

    /// Whether the effect can be delivered by a shape that only reaches the
    /// caster.
    pub const fn works_on_self(self) -> bool {
        matches!(
            self,
            Self::Heal | Self::Shield | Self::Teleport | Self::Damage | Self::Drain
        )
    }

    /// Whether the effect needs something other than the caster to land on.
    pub const fn needs_target(self) -> bool {
        matches!(self, Self::Slow | Self::Root)
    }
}

/// How much visual effort the client spends: a setting the player owns, and
/// what `crate::cast` scales particle counts by.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderQuality {
    Low,
    #[default]
    Medium,
    High,
}

impl RenderQuality {
    pub const ALL: [Self; 3] = [Self::Low, Self::Medium, Self::High];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|q| q.id() == id)
    }

    /// The next quality, cycling through all three: what the settings button
    /// does.
    pub const fn next(self) -> Self {
        match self {
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Low,
        }
    }

    /// How many particles one burst draws.
    pub const fn particle_count(self) -> u32 {
        match self {
            Self::Low => 4,
            Self::Medium => 8,
            Self::High => 16,
        }
    }
}

impl fmt::Display for RenderQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_ids_round_trip() {
        for effect in Effect::ALL {
            assert_eq!(Effect::from_id(effect.id()), Some(effect));
        }
    }

    #[test]
    fn qualities_cycle_and_round_trip() {
        for quality in RenderQuality::ALL {
            assert_eq!(RenderQuality::from_id(quality.id()), Some(quality));
        }
        assert_eq!(RenderQuality::Medium.next(), RenderQuality::High);
        assert_eq!(RenderQuality::High.next(), RenderQuality::Low);
        assert!(RenderQuality::High.particle_count() > RenderQuality::Low.particle_count());
    }

    #[test]
    fn only_self_effects_work_on_the_caster() {
        assert!(Effect::Heal.works_on_self());
        assert!(!Effect::Slow.works_on_self());
        assert!(Effect::Root.needs_target());
        assert!(!Effect::Damage.needs_target());
    }
}
