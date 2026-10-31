//! The grammar of a spell: the shape says how it reaches its target, the
//! effect says what happens when it lands.

/// How a spell is delivered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Shape {
    /// A single bolt at the nearest enemy in front of the caster.
    Projectile,
    /// A ray that hits the nearest enemy further away.
    Beam,
    /// A lingering area around the caster.
    Zone,
    /// A spell the caster turns on themselves.
    SelfCast,
    /// A burst that hits everything around the caster at once.
    Nova,
    /// A wall of magic in front of the caster: a wide, short-lived area.
    Wall,
    /// A step through space.
    Blink,
}

/// Who a shape can reach. `crate::cast` turns it into entity selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Targeting {
    /// The caster only.
    SelfOnly,
    /// One enemy near the caster.
    NearestEnemy,
    /// Every enemy around the caster.
    Area,
    /// The caster's own position, moved.
    Displacement,
}

impl Shape {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Projectile => "projectile",
            Self::Beam => "beam",
            Self::Zone => "zone",
            Self::SelfCast => "self",
            Self::Nova => "nova",
            Self::Wall => "wall",
            Self::Blink => "blink",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Projectile => "Projectile",
            Self::Beam => "Beam",
            Self::Zone => "Zone",
            Self::SelfCast => "Self",
            Self::Nova => "Nova",
            Self::Wall => "Wall",
            Self::Blink => "Blink",
        }
    }

    /// Every shape, in builder order.
    pub const ALL: [Shape; 7] = [
        Self::Projectile,
        Self::Beam,
        Self::Zone,
        Self::SelfCast,
        Self::Nova,
        Self::Wall,
        Self::Blink,
    ];

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.id() == id)
    }

    pub const fn targeting(self) -> Targeting {
        match self {
            Self::Projectile | Self::Beam => Targeting::NearestEnemy,
            Self::Zone | Self::Nova | Self::Wall => Targeting::Area,
            Self::SelfCast => Targeting::SelfOnly,
            Self::Blink => Targeting::Displacement,
        }
    }

    /// Mana the shape adds before the school's multiplier.
    pub const fn base_cost(self) -> f32 {
        match self {
            Self::Projectile => 10.0,
            Self::Beam => 12.0,
            Self::Zone => 14.0,
            Self::SelfCast => 6.0,
            Self::Nova => 18.0,
            Self::Wall => 16.0,
            Self::Blink => 8.0,
        }
    }

    /// Power the shape adds before the school's multiplier. A shape that
    /// reaches many targets carries less power per target.
    pub const fn base_power(self) -> f32 {
        match self {
            Self::Projectile => 8.0,
            Self::Beam => 7.0,
            Self::Zone => 5.0,
            Self::SelfCast => 4.0,
            Self::Nova => 10.0,
            Self::Wall => 4.0,
            Self::Blink => 0.0,
        }
    }

    /// How far it reaches, in blocks, before modifiers.
    pub const fn base_range(self) -> f32 {
        match self {
            Self::Projectile => 24.0,
            Self::Beam => 20.0,
            Self::Zone => 12.0,
            Self::SelfCast => 0.0,
            Self::Nova => 8.0,
            Self::Wall => 10.0,
            Self::Blink => 12.0,
        }
    }

    /// Ticks before the spell can be cast again, before power scaling.
    pub const fn base_cooldown(self) -> u32 {
        match self {
            Self::Projectile => 20,
            Self::Beam => 24,
            Self::Zone => 60,
            Self::SelfCast => 40,
            Self::Nova => 80,
            Self::Wall => 100,
            Self::Blink => 50,
        }
    }

    /// Whether a wall reaches wide rather than far: its area is twice its
    /// range across and half of it deep.
    pub const fn is_wall(self) -> bool {
        matches!(self, Self::Wall)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_round_trip() {
        for shape in Shape::ALL {
            assert_eq!(Shape::from_id(shape.id()), Some(shape));
        }
    }

    #[test]
    fn self_shapes_never_target_enemies() {
        assert_eq!(Shape::SelfCast.targeting(), Targeting::SelfOnly);
        assert_eq!(Shape::Blink.targeting(), Targeting::Displacement);
        for shape in [Shape::Projectile, Shape::Beam] {
            assert_eq!(shape.targeting(), Targeting::NearestEnemy);
        }
        for shape in [Shape::Zone, Shape::Nova, Shape::Wall] {
            assert_eq!(shape.targeting(), Targeting::Area);
        }
    }

    #[test]
    fn shapes_that_reach_further_cost_more() {
        assert!(Shape::Nova.base_cost() > Shape::Projectile.base_cost());
        assert!(Shape::SelfCast.base_cost() < Shape::Zone.base_cost());
        assert_eq!(Shape::SelfCast.base_range(), 0.0);
    }
}
