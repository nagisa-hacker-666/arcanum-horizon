//! Modifiers: what makes a spell yours rather than one of the six schools'
//! textbook casts.
//!
//! A modifier only ever shifts numbers a spell is derived from, which is what
//! lets `crate::cast` handle any combination without knowing about it.

/// One modifier a spell can carry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Modifier {
    /// The spell bursts on impact.
    Explosive,
    /// It jumps to a second target.
    Chaining,
    /// It goes through what it hits.
    Piercing,
    /// It costs less.
    Efficient,
    /// It reaches further.
    Extended,
    /// It recovers faster.
    Swift,
    /// It hits harder.
    Empowered,
    /// It heals the caster for part of its damage.
    Draining,
    /// It costs more but hits far harder.
    Overcharged,
}

impl Modifier {
    pub const ALL: [Modifier; 9] = [
        Self::Explosive,
        Self::Chaining,
        Self::Piercing,
        Self::Efficient,
        Self::Extended,
        Self::Swift,
        Self::Empowered,
        Self::Draining,
        Self::Overcharged,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Explosive => "explosive",
            Self::Chaining => "chaining",
            Self::Piercing => "piercing",
            Self::Efficient => "efficient",
            Self::Extended => "extended",
            Self::Swift => "swift",
            Self::Empowered => "empowered",
            Self::Draining => "draining",
            Self::Overcharged => "overcharged",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Explosive => "Explosive",
            Self::Chaining => "Chaining",
            Self::Piercing => "Piercing",
            Self::Efficient => "Efficient",
            Self::Extended => "Extended",
            Self::Swift => "Swift",
            Self::Empowered => "Empowered",
            Self::Draining => "Draining",
            Self::Overcharged => "Overcharged",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.id() == id)
    }

    /// Mana the modifier adds (or, when negative, takes away).
    pub const fn cost_delta(self) -> f32 {
        match self {
            Self::Explosive => 14.0,
            Self::Chaining => 16.0,
            Self::Piercing => 10.0,
            Self::Efficient => -8.0,
            Self::Extended => 8.0,
            Self::Swift => 6.0,
            Self::Empowered => 20.0,
            Self::Draining => 12.0,
            Self::Overcharged => 24.0,
        }
    }

    /// What the modifier multiplies the spell's power by, on top of 1.0.
    pub const fn power_bonus(self) -> f32 {
        match self {
            Self::Explosive => 0.45,
            Self::Chaining => 0.25,
            Self::Piercing => 0.20,
            Self::Efficient => -0.15,
            Self::Extended => 0.0,
            Self::Swift => -0.10,
            Self::Empowered => 0.55,
            Self::Draining => 0.10,
            Self::Overcharged => 0.95,
        }
    }

    /// What the modifier multiplies the spell's cost by, on top of 1.0.
    pub const fn cost_scale(self) -> f32 {
        match self {
            Self::Overcharged => 1.20,
            Self::Efficient => 0.85,
            _ => 1.0,
        }
    }

    /// What the modifier multiplies the spell's range by, on top of 1.0.
    pub const fn range_scale(self) -> f32 {
        match self {
            Self::Extended => 1.5,
            Self::Piercing => 1.2,
            _ => 1.0,
        }
    }

    /// Ticks the modifier adds to (or takes off) the cooldown.
    pub const fn cooldown_delta(self) -> i32 {
        match self {
            Self::Swift => -12,
            Self::Overcharged => 40,
            Self::Empowered => 20,
            Self::Chaining => 10,
            _ => 0,
        }
    }

    /// Whether two modifiers contradict each other. Cheap-and-weak cannot be
    /// stacked with expensive-and-strong: that is what makes the efficiency
    /// floor hold.
    pub fn conflicts_with(self, other: Self) -> bool {
        let pair = |a: Self, b: Self| (self == a && other == b) || (self == b && other == a);
        pair(Self::Efficient, Self::Empowered)
            || pair(Self::Efficient, Self::Overcharged)
            || pair(Self::Swift, Self::Overcharged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip() {
        for modifier in Modifier::ALL {
            assert_eq!(Modifier::from_id(modifier.id()), Some(modifier));
        }
    }

    #[test]
    fn conflicts_are_symmetric_and_irreflexive() {
        for a in Modifier::ALL {
            assert!(!a.conflicts_with(a), "{} conflicts with itself", a.id());
            for b in Modifier::ALL {
                assert_eq!(
                    a.conflicts_with(b),
                    b.conflicts_with(a),
                    "{}/{}",
                    a.id(),
                    b.id()
                );
            }
        }
    }

    #[test]
    fn cheap_and_strong_are_mutually_exclusive() {
        assert!(Modifier::Efficient.conflicts_with(Modifier::Empowered));
        assert!(Modifier::Efficient.conflicts_with(Modifier::Overcharged));
        assert!(!Modifier::Efficient.conflicts_with(Modifier::Extended));
    }
}
