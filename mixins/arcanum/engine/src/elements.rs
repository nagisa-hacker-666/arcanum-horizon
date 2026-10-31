//! The six schools of magic.
//!
//! Every school carries its own identity: a HUD colour, the vanilla particle
//! and sound a cast uses, and the vanilla damage type its damage is dealt as.
//! Only identifiers that exist in vanilla 26.3 are used, because the effects
//! are executed by the authoritative server as vanilla commands
//! (`crate::cast`).

/// One of Arcanum's six schools.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Element {
    Pyro,
    Cryo,
    Electro,
    Nature,
    Shadow,
    Arcane,
}

impl Element {
    /// Every school, in HUD order.
    pub const ALL: [Element; 6] = [
        Element::Pyro,
        Element::Cryo,
        Element::Electro,
        Element::Nature,
        Element::Shadow,
        Element::Arcane,
    ];

    /// The school's stable id, used on disk, in commands and in mod messages.
    pub const fn id(self) -> &'static str {
        match self {
            Self::Pyro => "pyro",
            Self::Cryo => "cryo",
            Self::Electro => "electro",
            Self::Nature => "nature",
            Self::Shadow => "shadow",
            Self::Arcane => "arcane",
        }
    }

    /// The school's name, as the HUD and the chat show it.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pyro => "Pyromancy",
            Self::Cryo => "Cryomancy",
            Self::Electro => "Electromancy",
            Self::Nature => "Nature Magic",
            Self::Shadow => "Shadow Magic",
            Self::Arcane => "Arcane Magic",
        }
    }

    /// The school's colour in the HUD, as `0xRRGGBB`.
    pub const fn color(self) -> u32 {
        match self {
            Self::Pyro => 0xFF7A2F,
            Self::Cryo => 0x7FD8FF,
            Self::Electro => 0xFFE96B,
            Self::Nature => 0x7CE87C,
            Self::Shadow => 0xB07CFF,
            Self::Arcane => 0x6FC3FF,
        }
    }

    /// The vanilla particle a cast of this school draws.
    pub const fn particle(self) -> &'static str {
        match self {
            Self::Pyro => "minecraft:flame",
            Self::Cryo => "minecraft:snowflake",
            Self::Electro => "minecraft:electric_spark",
            Self::Nature => "minecraft:composter",
            Self::Shadow => "minecraft:squid_ink",
            Self::Arcane => "minecraft:end_rod",
        }
    }

    /// The vanilla sound a cast of this school plays.
    pub const fn sound(self) -> &'static str {
        match self {
            Self::Pyro => "minecraft:entity.blaze.shoot",
            Self::Cryo => "minecraft:block.glass.break",
            Self::Electro => "minecraft:entity.lightning_bolt.thunder",
            Self::Nature => "minecraft:block.grass.break",
            Self::Shadow => "minecraft:entity.warden.sonic_boom",
            Self::Arcane => "minecraft:block.amethyst_block.chime",
        }
    }

    /// The vanilla damage type this school's damage is dealt as.
    pub const fn damage_type(self) -> &'static str {
        match self {
            Self::Pyro => "minecraft:on_fire",
            Self::Cryo => "minecraft:freeze",
            Self::Electro => "minecraft:lightning_bolt",
            Self::Nature => "minecraft:magic",
            Self::Shadow => "minecraft:wither",
            Self::Arcane => "minecraft:indirect_magic",
        }
    }

    /// What a spell of this school costs, relative to the bare shape and
    /// effect. Fire and shadow are cheap; arcane is expensive.
    pub const fn cost_multiplier(self) -> f32 {
        match self {
            Self::Pyro => 1.00,
            Self::Cryo => 1.00,
            Self::Electro => 1.15,
            Self::Nature => 0.90,
            Self::Shadow => 1.10,
            Self::Arcane => 1.25,
        }
    }

    /// What a spell of this school hits for, relative to the bare shape and
    /// effect. Pyromancy hits hardest; nature magic is the gentlest.
    pub const fn power_multiplier(self) -> f32 {
        match self {
            Self::Pyro => 1.10,
            Self::Cryo => 0.95,
            Self::Electro => 1.05,
            Self::Nature => 0.85,
            Self::Shadow => 1.00,
            Self::Arcane => 1.10,
        }
    }

    /// The school whose id is `id`, as a command or a save names it.
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.id() == id)
    }

    /// The school's index in [`Element::ALL`], which is how mastery is stored.
    pub const fn index(self) -> usize {
        match self {
            Self::Pyro => 0,
            Self::Cryo => 1,
            Self::Electro => 2,
            Self::Nature => 3,
            Self::Shadow => 4,
            Self::Arcane => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_and_are_unique() {
        let mut seen = Vec::new();
        for element in Element::ALL {
            assert_eq!(Element::from_id(element.id()), Some(element));
            assert!(
                !seen.contains(&element.id()),
                "duplicate id {}",
                element.id()
            );
            seen.push(element.id());
        }
    }

    #[test]
    fn every_visual_identity_is_a_vanilla_identifier() {
        for element in Element::ALL {
            assert!(
                element.particle().starts_with("minecraft:"),
                "{}",
                element.particle()
            );
            assert!(
                element.sound().starts_with("minecraft:"),
                "{}",
                element.sound()
            );
            assert!(
                element.damage_type().starts_with("minecraft:"),
                "{}",
                element.damage_type()
            );
            assert!(
                element.sound().contains('.'),
                "{} is not a sound event",
                element.sound()
            );
        }
    }

    #[test]
    fn colours_are_opaque_rgb() {
        for element in Element::ALL {
            assert_ne!(element.color() & 0xFF00_0000, 0xFF00_0000);
            assert_ne!(element.color() & 0x00FF_FFFF, 0);
        }
    }
}
