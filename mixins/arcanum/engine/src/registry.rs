//! The spells Arcanum ships with: five per school, thirty in all.
//!
//! The table below is the whole content of the mod's spellbook. Every entry
//! goes through [`SpellBuilder`], so a typo in a number is caught by
//! `tests/balance_tests.rs` instead of shipping an unbalanced spell.

use crate::balance::Limits;
use crate::effects::Effect;
use crate::elements::Element;
use crate::modifiers::Modifier;
use crate::shapes::Shape;
use crate::spell::Spell;
use crate::spell_builder::{BuildError, SpellBuilder};

/// One line of the spellbook.
struct Def {
    id: &'static str,
    name: &'static str,
    element: Element,
    shape: Shape,
    effect: Effect,
    modifiers: &'static [Modifier],
    level: u32,
}

const DEFS: [Def; 30] = [
    // Pyromancy
    Def {
        id: "pyro_fireball",
        name: "Fireball",
        element: Element::Pyro,
        shape: Shape::Projectile,
        effect: Effect::Damage,
        modifiers: &[Modifier::Explosive],
        level: 1,
    },
    Def {
        id: "pyro_wall_of_flames",
        name: "Wall of Flames",
        element: Element::Pyro,
        shape: Shape::Wall,
        effect: Effect::Slow,
        modifiers: &[Modifier::Extended],
        level: 5,
    },
    Def {
        id: "pyro_volcanic_burst",
        name: "Volcanic Burst",
        element: Element::Pyro,
        shape: Shape::Nova,
        effect: Effect::Damage,
        modifiers: &[Modifier::Explosive],
        level: 8,
    },
    Def {
        id: "pyro_blazing_nova",
        name: "Blazing Nova",
        element: Element::Pyro,
        shape: Shape::Nova,
        effect: Effect::Damage,
        modifiers: &[Modifier::Empowered],
        level: 14,
    },
    Def {
        id: "pyro_meteor",
        name: "Meteor",
        element: Element::Pyro,
        shape: Shape::Beam,
        effect: Effect::Damage,
        modifiers: &[Modifier::Explosive, Modifier::Overcharged],
        level: 20,
    },
    // Cryomancy
    Def {
        id: "cryo_ice_bolt",
        name: "Ice Bolt",
        element: Element::Cryo,
        shape: Shape::Projectile,
        effect: Effect::Damage,
        modifiers: &[Modifier::Piercing],
        level: 1,
    },
    Def {
        id: "cryo_snowstorm",
        name: "Snowstorm",
        element: Element::Cryo,
        shape: Shape::Zone,
        effect: Effect::Slow,
        modifiers: &[Modifier::Extended],
        level: 6,
    },
    Def {
        id: "cryo_crystal_wall",
        name: "Crystal Wall",
        element: Element::Cryo,
        shape: Shape::Wall,
        effect: Effect::Root,
        modifiers: &[Modifier::Extended],
        level: 10,
    },
    Def {
        id: "cryo_ice_prison",
        name: "Ice Prison",
        element: Element::Cryo,
        shape: Shape::Zone,
        effect: Effect::Root,
        modifiers: &[Modifier::Extended],
        level: 12,
    },
    Def {
        id: "cryo_blizzard",
        name: "Blizzard",
        element: Element::Cryo,
        shape: Shape::Nova,
        effect: Effect::Slow,
        modifiers: &[Modifier::Extended, Modifier::Swift],
        level: 22,
    },
    // Electromancy
    Def {
        id: "electro_bolt",
        name: "Lightning Bolt",
        element: Element::Electro,
        shape: Shape::Beam,
        effect: Effect::Damage,
        modifiers: &[],
        level: 1,
    },
    Def {
        id: "electro_blink",
        name: "Storm Step",
        element: Element::Electro,
        shape: Shape::Blink,
        effect: Effect::Teleport,
        modifiers: &[],
        level: 4,
    },
    Def {
        id: "electro_chain",
        name: "Chain Lightning",
        element: Element::Electro,
        shape: Shape::Beam,
        effect: Effect::Damage,
        modifiers: &[Modifier::Chaining],
        level: 10,
    },
    Def {
        id: "electro_overload",
        name: "Overload",
        element: Element::Electro,
        shape: Shape::SelfCast,
        effect: Effect::Shield,
        modifiers: &[Modifier::Overcharged],
        level: 18,
    },
    Def {
        id: "electro_storm",
        name: "Thunderstorm",
        element: Element::Electro,
        shape: Shape::Nova,
        effect: Effect::Damage,
        modifiers: &[Modifier::Chaining, Modifier::Extended],
        level: 26,
    },
    // Nature magic
    Def {
        id: "nature_roots",
        name: "Grasping Roots",
        element: Element::Nature,
        shape: Shape::Zone,
        effect: Effect::Root,
        modifiers: &[],
        level: 1,
    },
    Def {
        id: "nature_heal",
        name: "Natural Healing",
        element: Element::Nature,
        shape: Shape::SelfCast,
        effect: Effect::Heal,
        modifiers: &[],
        level: 1,
    },
    Def {
        id: "nature_thorn_wall",
        name: "Thorn Wall",
        element: Element::Nature,
        shape: Shape::Wall,
        effect: Effect::Slow,
        modifiers: &[Modifier::Extended],
        level: 8,
    },
    Def {
        id: "nature_guardian",
        name: "Guardian Spirit",
        element: Element::Nature,
        shape: Shape::SelfCast,
        effect: Effect::Shield,
        modifiers: &[Modifier::Empowered],
        level: 16,
    },
    Def {
        id: "nature_sanctuary",
        name: "Healing Sanctuary",
        element: Element::Nature,
        shape: Shape::Zone,
        effect: Effect::Heal,
        modifiers: &[Modifier::Extended, Modifier::Efficient],
        level: 20,
    },
    // Shadow magic
    Def {
        id: "shadow_blade",
        name: "Spectral Blade",
        element: Element::Shadow,
        shape: Shape::Projectile,
        effect: Effect::Damage,
        modifiers: &[Modifier::Piercing],
        level: 1,
    },
    Def {
        id: "shadow_step",
        name: "Shadow Step",
        element: Element::Shadow,
        shape: Shape::Blink,
        effect: Effect::Teleport,
        modifiers: &[Modifier::Swift],
        level: 6,
    },
    Def {
        id: "shadow_drain",
        name: "Vampiric Orb",
        element: Element::Shadow,
        shape: Shape::Projectile,
        effect: Effect::Drain,
        modifiers: &[Modifier::Draining],
        level: 10,
    },
    Def {
        id: "shadow_clone",
        name: "Umbral Clone",
        element: Element::Shadow,
        shape: Shape::Nova,
        effect: Effect::Slow,
        modifiers: &[Modifier::Draining],
        level: 22,
    },
    Def {
        id: "shadow_rift",
        name: "Abyssal Rift",
        element: Element::Shadow,
        shape: Shape::Beam,
        effect: Effect::Damage,
        modifiers: &[Modifier::Explosive, Modifier::Overcharged],
        level: 30,
    },
    // Arcane magic
    Def {
        id: "arcane_ray",
        name: "Arcane Ray",
        element: Element::Arcane,
        shape: Shape::Beam,
        effect: Effect::Damage,
        modifiers: &[],
        level: 1,
    },
    Def {
        id: "arcane_shield",
        name: "Arcane Shield",
        element: Element::Arcane,
        shape: Shape::SelfCast,
        effect: Effect::Shield,
        modifiers: &[],
        level: 5,
    },
    Def {
        id: "arcane_distortion",
        name: "Spatial Distortion",
        element: Element::Arcane,
        shape: Shape::Blink,
        effect: Effect::Teleport,
        modifiers: &[Modifier::Extended],
        level: 12,
    },
    Def {
        id: "arcane_dimension_burst",
        name: "Dimensional Burst",
        element: Element::Arcane,
        shape: Shape::Nova,
        effect: Effect::Damage,
        modifiers: &[Modifier::Explosive, Modifier::Overcharged],
        level: 34,
    },
    Def {
        id: "arcane_singularity",
        name: "Singularity",
        element: Element::Arcane,
        shape: Shape::Zone,
        effect: Effect::Damage,
        modifiers: &[Modifier::Empowered, Modifier::Explosive, Modifier::Chaining],
        level: 40,
    },
];

/// The spells a world knows.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpellRegistry {
    spells: Vec<Spell>,
}

impl SpellRegistry {
    /// A registry with nothing in it: what a mage has when the spellbook could
    /// not be built, so the game keeps running.
    pub const fn empty() -> Self {
        Self { spells: Vec::new() }
    }

    /// Builds the spellbook Arcanum ships with.
    pub fn builtin() -> Result<Self, BuildError> {
        let limits = Limits::default();
        let mut spells = Vec::with_capacity(DEFS.len());
        for def in &DEFS {
            let spell = SpellBuilder::new(
                &limits,
                def.id,
                def.name,
                def.element,
                def.shape,
                def.effect,
            )
            .modifiers(def.modifiers)
            .required_level(def.level)
            .build()?;
            spells.push(spell);
        }
        Ok(Self { spells })
    }

    pub fn get(&self, id: &str) -> Option<&Spell> {
        self.spells.iter().find(|s| s.id == id)
    }

    pub fn all(&self) -> &[Spell] {
        &self.spells
    }

    pub fn len(&self) -> usize {
        self.spells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.spells.is_empty()
    }

    /// Every spell of one school, in spellbook order.
    pub fn by_school(&self, element: Element) -> impl Iterator<Item = &Spell> {
        self.spells.iter().filter(move |s| s.element == element)
    }

    /// The spells a mage of `level` may learn.
    pub fn learnable(&self, level: u32) -> impl Iterator<Item = &Spell> {
        self.spells
            .iter()
            .filter(move |s| s.required_level <= level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_spellbook_builds() {
        let registry = SpellRegistry::builtin().expect("balanced");
        assert_eq!(registry.len(), 30);
        for element in Element::ALL {
            assert_eq!(registry.by_school(element).count(), 5, "{}", element.id());
        }
    }

    #[test]
    fn ids_are_unique() {
        let registry = SpellRegistry::builtin().expect("balanced");
        for (index, spell) in registry.all().iter().enumerate() {
            let later = &registry.all()[index + 1..];
            assert!(
                !later.iter().any(|other| other.id == spell.id),
                "duplicate id {}",
                spell.id
            );
        }
    }

    #[test]
    fn an_empty_book_answers_nothing() {
        let registry = SpellRegistry::empty();
        assert!(registry.is_empty());
        assert!(registry.get("pyro_fireball").is_none());
        assert_eq!(registry.learnable(50).count(), 0);
    }
}
