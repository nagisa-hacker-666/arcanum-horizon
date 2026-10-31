//! The modular spell builder: the feature Arcanum exists for.
//!
//! A spell is an element, a shape, an effect and up to
//! [`Limits::max_modifiers`] modifiers. The builder collects them and
//! [`SpellBuilder::build`] refuses anything that breaks the economy, so a
//! player can never compose a spell the server would have to special-case.

use std::fmt;

use crate::balance::Limits;
use crate::effects::Effect;
use crate::elements::Element;
use crate::modifiers::Modifier;
use crate::shapes::Shape;
use crate::spell::{Spell, SpellSpec};

/// Why a composed spell was refused.
#[derive(Clone, Debug, PartialEq)]
pub enum BuildError {
    Unnamed,
    TooManyModifiers {
        count: usize,
        max: usize,
    },
    DuplicateModifier(&'static str),
    ConflictingModifiers(&'static str, &'static str),
    IncompatiblePair(&'static str, &'static str),
    CostTooHigh {
        cost: f32,
        max: f32,
    },
    PowerTooHigh {
        power: f32,
        max: f32,
    },
    RangeTooHigh {
        range: f32,
        max: f32,
    },
    CooldownTooLong {
        cooldown: u32,
        max: u32,
    },
    /// A spell cheaper than [`crate::balance::MIN_COST_PER_POWER`] times its
    /// own power: the combination everybody would otherwise cast.
    TooEfficient {
        cost: f32,
        power: f32,
    },
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unnamed => f.write_str("a spell needs an id and a name"),
            Self::TooManyModifiers { count, max } => {
                write!(f, "{count} modifiers is more than the {max} allowed")
            }
            Self::DuplicateModifier(id) => write!(f, "the modifier `{id}` is already on the spell"),
            Self::ConflictingModifiers(a, b) => {
                write!(f, "`{a}` and `{b}` cannot be on the same spell")
            }
            Self::IncompatiblePair(shape, effect) => {
                write!(f, "the shape `{shape}` cannot carry the effect `{effect}`")
            }
            Self::CostTooHigh { cost, max } => {
                write!(f, "it costs {cost:.1} mana, more than the {max:.1} allowed")
            }
            Self::PowerTooHigh { power, max } => {
                write!(f, "its power is {power:.1}, more than the {max:.1} allowed")
            }
            Self::RangeTooHigh { range, max } => {
                write!(f, "it reaches {range:.1} blocks, more than the {max:.1} allowed")
            }
            Self::CooldownTooLong { cooldown, max } => {
                write!(f, "its cooldown is {cooldown} ticks, more than the {max} allowed")
            }
            Self::TooEfficient { cost, power } => write!(
                f,
                "it costs {cost:.1} mana for {power:.1} power: at least {:.1} mana per point of power is required",
                crate::balance::MIN_COST_PER_POWER
            ),
        }
    }
}

impl std::error::Error for BuildError {}

/// Composes a spell.
///
/// `modifier` never fails on the spot: it records what was asked for and
/// [`build`](Self::build) refuses the combination, so the error names the
/// conflict rather than the call that made it.
#[derive(Clone, Debug)]
pub struct SpellBuilder<'a> {
    limits: &'a Limits,
    id: &'a str,
    name: &'a str,
    element: Element,
    shape: Shape,
    effect: Effect,
    modifiers: Vec<Modifier>,
    required_level: u32,
}

impl<'a> SpellBuilder<'a> {
    /// Starts a spell with the six things it cannot change later.
    pub fn new(
        limits: &'a Limits,
        id: &'a str,
        name: &'a str,
        element: Element,
        shape: Shape,
        effect: Effect,
    ) -> Self {
        Self {
            limits,
            id,
            name,
            element,
            shape,
            effect,
            modifiers: Vec::new(),
            required_level: 1,
        }
    }

    /// Adds a modifier.
    pub fn modifier(mut self, modifier: Modifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    /// Adds several modifiers at once, as the spellbook declares them.
    pub fn modifiers<'m>(mut self, modifiers: impl IntoIterator<Item = &'m Modifier>) -> Self {
        for modifier in modifiers {
            self.modifiers.push(*modifier);
        }
        self
    }

    /// Sets the mage level required to learn the spell.
    pub fn required_level(mut self, level: u32) -> Self {
        self.required_level = level;
        self
    }

    /// Works the numbers out and checks every rule.
    pub fn build(self) -> Result<Spell, BuildError> {
        Spell::derive(
            SpellSpec {
                id: self.id,
                name: self.name,
                element: self.element,
                shape: self.shape,
                effect: self.effect,
                modifiers: &self.modifiers,
                required_level: self.required_level,
            },
            self.limits,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builder<'a>(limits: &'a Limits) -> SpellBuilder<'a> {
        SpellBuilder::new(
            limits,
            "test",
            "Test",
            Element::Pyro,
            Shape::Projectile,
            Effect::Damage,
        )
    }

    #[test]
    fn a_plain_spell_builds() {
        let limits = Limits::default();
        let spell = builder(&limits).build().expect("balanced");
        assert!(spell.modifiers.is_empty());
        assert_eq!(spell.required_level, 1);
    }

    #[test]
    fn duplicate_modifiers_are_refused() {
        let limits = Limits::default();
        let err = builder(&limits)
            .modifier(Modifier::Explosive)
            .modifier(Modifier::Explosive)
            .build();
        assert_eq!(err, Err(BuildError::DuplicateModifier("explosive")));
    }

    #[test]
    fn conflicts_are_refused_by_name() {
        let limits = Limits::default();
        let err = builder(&limits)
            .modifier(Modifier::Efficient)
            .modifier(Modifier::Empowered)
            .build();
        assert_eq!(
            err,
            Err(BuildError::ConflictingModifiers("efficient", "empowered"))
        );
    }

    #[test]
    fn too_many_modifiers_are_refused() {
        let limits = Limits::default();
        let err = builder(&limits)
            .modifier(Modifier::Explosive)
            .modifier(Modifier::Chaining)
            .modifier(Modifier::Piercing)
            .modifier(Modifier::Extended)
            .build();
        assert!(matches!(
            err,
            Err(BuildError::TooManyModifiers { count: 4, max: 3 })
        ));
    }

    #[test]
    fn a_ceiling_refuses_an_expensive_spell() {
        let limits = Limits::default().with_max_cost(20.0);
        let err = builder(&limits).modifier(Modifier::Explosive).build();
        assert!(matches!(err, Err(BuildError::CostTooHigh { .. })));
    }

    #[test]
    fn modifiers_from_a_slice_are_all_kept() {
        let limits = Limits::default();
        let spell = builder(&limits)
            .modifiers(&[Modifier::Explosive, Modifier::Piercing])
            .build()
            .expect("balanced");
        assert_eq!(spell.modifiers.len(), 2);
        assert!(spell.has(Modifier::Piercing));
    }

    #[test]
    fn errors_say_what_to_change() {
        let text = BuildError::TooEfficient {
            cost: 10.0,
            power: 30.0,
        }
        .to_string();
        assert!(text.contains("10.0") && text.contains("30.0"));
    }
}
