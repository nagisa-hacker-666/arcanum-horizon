//! A composed spell: the eight things every Arcanum spell has.

use crate::balance::{Limits, DAMAGE_PER_POWER, MIN_COST_PER_POWER};
use crate::effects::Effect;
use crate::elements::Element;
use crate::modifiers::Modifier;
use crate::shapes::{Shape, Targeting};
use crate::spell_builder::BuildError;

/// The six things a spell is made of, before its numbers are derived.
#[derive(Clone, Copy, Debug)]
pub struct SpellSpec<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub element: Element,
    pub shape: Shape,
    pub effect: Effect,
    pub modifiers: &'a [Modifier],
    pub required_level: u32,
}

/// A spell, with every derived number already worked out.
///
/// Spells are immutable once built: the builder is the only way to make one,
/// and it is what enforces the caps in [`Limits`].
#[derive(Clone, Debug, PartialEq)]
pub struct Spell {
    /// Stable id: what a save, a command and a mod message name.
    pub id: String,
    /// What the HUD shows.
    pub name: String,
    pub element: Element,
    pub shape: Shape,
    pub effect: Effect,
    pub modifiers: Vec<Modifier>,
    /// Mana it costs before the caster's own reductions.
    pub cost: f32,
    /// Its power before the caster's own bonuses.
    pub power: f32,
    /// How far it reaches, in blocks.
    pub range: f32,
    /// Ticks before it may be cast again, before the caster's own reductions.
    pub cooldown: u32,
    /// The mage level a mage must reach to learn it.
    pub required_level: u32,
}

impl Spell {
    /// Derives a spell's numbers from its parts and refuses it when it breaks
    /// a rule of [`Limits`].
    pub fn derive(spec: SpellSpec<'_>, limits: &Limits) -> Result<Self, BuildError> {
        let SpellSpec {
            id,
            name,
            element,
            shape,
            effect,
            modifiers,
            required_level,
        } = spec;
        if id.trim().is_empty() || name.trim().is_empty() {
            return Err(BuildError::Unnamed);
        }
        if modifiers.len() > limits.max_modifiers {
            return Err(BuildError::TooManyModifiers {
                count: modifiers.len(),
                max: limits.max_modifiers,
            });
        }
        for (index, modifier) in modifiers.iter().enumerate() {
            if modifiers[..index].contains(modifier) {
                return Err(BuildError::DuplicateModifier(modifier.id()));
            }
            for other in &modifiers[index + 1..] {
                if modifier.conflicts_with(*other) {
                    return Err(BuildError::ConflictingModifiers(modifier.id(), other.id()));
                }
            }
        }
        Self::check_shape_and_effect(shape, effect)?;

        let power_scale: f32 = modifiers.iter().map(|m| 1.0 + m.power_bonus()).product();
        let cost_scale: f32 = modifiers.iter().map(|m| m.cost_scale()).product();
        let range_scale: f32 = modifiers.iter().map(|m| m.range_scale()).product();

        let base_cost = (shape.base_cost() + effect.base_cost()) * element.cost_multiplier();
        let added_cost: f32 = modifiers.iter().map(|m| m.cost_delta()).sum();
        let cost = round1(((base_cost + added_cost).max(1.0) * cost_scale).max(1.0));
        let power = round1(
            (shape.base_power() + effect.base_power()) * element.power_multiplier() * power_scale,
        );
        let range = round1(shape.base_range() * range_scale);
        let cooldown = Self::cooldown_for(shape, power, modifiers, limits);

        if cost > limits.max_cost {
            return Err(BuildError::CostTooHigh {
                cost,
                max: limits.max_cost,
            });
        }
        if power > limits.max_power {
            return Err(BuildError::PowerTooHigh {
                power,
                max: limits.max_power,
            });
        }
        if range > limits.max_range {
            return Err(BuildError::RangeTooHigh {
                range,
                max: limits.max_range,
            });
        }
        if cooldown > limits.max_cooldown {
            return Err(BuildError::CooldownTooLong {
                cooldown,
                max: limits.max_cooldown,
            });
        }
        if cost < power * MIN_COST_PER_POWER {
            return Err(BuildError::TooEfficient { cost, power });
        }

        Ok(Self {
            id: id.trim().to_owned(),
            name: name.trim().to_owned(),
            element,
            shape,
            effect,
            modifiers: modifiers.to_vec(),
            cost,
            power,
            range,
            cooldown,
            required_level,
        })
    }

    /// A shape and an effect must agree on who they reach: a self-only shape
    /// cannot slow an enemy, and only a blink teleports.
    fn check_shape_and_effect(shape: Shape, effect: Effect) -> Result<(), BuildError> {
        let self_only = shape.targeting() == Targeting::SelfOnly;
        if self_only && !effect.works_on_self() {
            return Err(BuildError::IncompatiblePair(shape.id(), effect.id()));
        }
        if effect == Effect::Teleport && shape != Shape::Blink {
            return Err(BuildError::IncompatiblePair(shape.id(), effect.id()));
        }
        if shape == Shape::Blink && effect != Effect::Teleport {
            return Err(BuildError::IncompatiblePair(shape.id(), effect.id()));
        }
        // Healing and shielding land on the caster or on everyone around them:
        // a bolt that heals is a bolt that heals whoever it was aimed at, which
        // is not what "healing" means.
        if matches!(effect, Effect::Heal | Effect::Shield)
            && !matches!(shape.targeting(), Targeting::SelfOnly | Targeting::Area)
        {
            return Err(BuildError::IncompatiblePair(shape.id(), effect.id()));
        }
        Ok(())
    }

    /// The cooldown formula: the shape's own recovery, what the modifiers add
    /// or take away, and a share of the shape's recovery again in proportion
    /// to how hard the spell hits.
    fn cooldown_for(shape: Shape, power: f32, modifiers: &[Modifier], limits: &Limits) -> u32 {
        let base = shape.base_cooldown() as f32;
        let added: i32 = modifiers.iter().map(|m| m.cooldown_delta()).sum();
        let scaled = base * (1.0 + power / limits.max_power * 0.5);
        let ticks = (scaled + added as f32).round().max(0.0) as u32;
        ticks.clamp(limits.min_cooldown, limits.max_cooldown)
    }

    /// Health a landed cast of a damaging spell takes.
    pub fn damage(&self) -> f32 {
        round1(self.power * DAMAGE_PER_POWER)
    }

    /// Magic experience a successful cast of this spell is worth.
    pub fn xp_reward(&self) -> u64 {
        (self.cost * 2.0).round() as u64
    }

    /// Whether the spell carries a modifier.
    pub fn has(&self, modifier: Modifier) -> bool {
        self.modifiers.contains(&modifier)
    }

    /// One line for the HUD, the grimoire and the chat.
    pub fn describe(&self) -> String {
        let mut text = format!(
            "{} — {} {} with {}",
            self.name,
            self.element.name(),
            self.shape.name().to_lowercase(),
            self.effect.name().to_lowercase()
        );
        if !self.modifiers.is_empty() {
            let names: Vec<&str> = self.modifiers.iter().map(|m| m.name()).collect();
            text.push_str(" (");
            text.push_str(&names.join(", "));
            text.push(')');
        }
        text
    }
}

/// Rounds to one decimal, the precision vanilla commands and the HUD use.
pub(crate) fn round1(value: f32) -> f32 {
    (value * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec<'a>(shape: Shape, effect: Effect, modifiers: &'a [Modifier]) -> SpellSpec<'a> {
        SpellSpec {
            id: "test",
            name: "Test",
            element: Element::Pyro,
            shape,
            effect,
            modifiers,
            required_level: 1,
        }
    }

    #[test]
    fn derived_numbers_follow_the_formula() {
        let limits = Limits::default();
        let spell = Spell::derive(
            spec(Shape::Projectile, Effect::Damage, &[Modifier::Explosive]),
            &limits,
        )
        .expect("balanced");
        // (10 + 4) * 1.0 + 14 = 28
        assert_eq!(spell.cost, 28.0);
        // (8 + 6) * 1.1 * 1.45 = 22.33
        assert_eq!(spell.power, 22.3);
        assert_eq!(spell.range, 24.0);
        assert!(spell.cooldown >= limits.min_cooldown);
        assert_eq!(spell.damage(), 12.3);
    }

    #[test]
    fn a_self_shape_cannot_slow() {
        let limits = Limits::default();
        let err = Spell::derive(spec(Shape::SelfCast, Effect::Slow, &[]), &limits);
        assert!(matches!(err, Err(BuildError::IncompatiblePair(..))));
    }

    #[test]
    fn only_a_blink_teleports() {
        let limits = Limits::default();
        assert!(Spell::derive(spec(Shape::Projectile, Effect::Teleport, &[]), &limits).is_err());
        assert!(Spell::derive(spec(Shape::Blink, Effect::Damage, &[]), &limits).is_err());
    }

    #[test]
    fn rounding_is_to_one_decimal() {
        assert_eq!(round1(1.24), 1.2);
        assert_eq!(round1(1.25), 1.3);
        assert_eq!(round1(-0.04), -0.0);
    }
}
