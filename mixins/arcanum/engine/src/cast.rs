//! Turning a cast into what the authoritative server does about it.
//!
//! Arcanum has no rendering hook of its own, so a cast resolves into vanilla
//! commands the server runs as its console (`Game::native_run_command`, hook
//! API 21). Particle and sound commands keep each school's visual identity on
//! any Horizon build, and because the server produces them every nearby player
//! sees the same spell.
//!
//! Nothing here trusts the client: the caller passes a [`Spell`] the server
//! already looked up in its own registry, and the caster's name, which the
//! server took from the player it is acting for.

use std::fmt;

use crate::balance::DAMAGE_PER_POWER;
use crate::effects::{Effect, RenderQuality};
use crate::shapes::{Shape, Targeting};
use crate::spell::{round1, Spell};

/// Why a cast did not happen.
#[derive(Clone, Debug, PartialEq)]
pub enum CastError {
    /// The player's name cannot go into an entity selector.
    InvalidOrigin,
    /// Nothing is selected in the spell wheel.
    EmptySelection,
    /// The selected spell is not in the registry any more.
    UnknownSpell(String),
    /// The mage never learned it.
    NotLearned(String),
    NotEnoughMana {
        needed: f32,
        available: f32,
    },
    OnCooldown {
        ticks: u32,
    },
}

impl fmt::Display for CastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOrigin => f.write_str("the caster has no usable name"),
            Self::EmptySelection => f.write_str("no spell is selected"),
            Self::UnknownSpell(id) => write!(f, "no spell is called `{id}`"),
            Self::NotLearned(id) => write!(f, "`{id}` is not in your grimoire"),
            Self::NotEnoughMana { needed, available } => {
                write!(f, "needs {needed:.1} mana, you have {available:.1}")
            }
            Self::OnCooldown { ticks } => {
                write!(f, "still cooling down for {:.1}s", *ticks as f32 / 20.0)
            }
        }
    }
}

impl std::error::Error for CastError {}

/// What the server does for one cast: the commands, in the order they must
/// run, and the line the caster reads back.
#[derive(Clone, Debug, PartialEq)]
pub struct EffectPlan {
    pub commands: Vec<String>,
    pub feedback: String,
}

impl EffectPlan {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            feedback: String::new(),
        }
    }
}

impl Default for EffectPlan {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolves `spell`, cast by `origin`, at an effective `power` (the spell's
/// own power after the caster's bonuses).
pub fn resolve(
    spell: &Spell,
    origin: &str,
    power: f32,
    quality: RenderQuality,
) -> Result<EffectPlan, CastError> {
    if !valid_origin(origin) {
        return Err(CastError::InvalidOrigin);
    }
    let power = round1(power.max(0.0));
    let damage = round1(power * DAMAGE_PER_POWER);
    let reach = spell.range.max(1.0);
    let radius = (spell.range * 0.5).max(2.0);
    let amplifier = (power / 12.0).floor().clamp(0.0, 4.0) as i32;
    let seconds = (power / 4.0).ceil().clamp(1.0, 60.0) as i32;
    let nearest = format!("@e[distance=..{reach:.1},sort=nearest,limit=1,type=!minecraft:player]");
    let area = format!("@e[distance=..{radius:.1},type=!minecraft:player]");
    let damage_type = spell.element.damage_type();

    let mut plan = EffectPlan::new();
    match spell.effect {
        Effect::Damage | Effect::Drain => {
            if let Some(selector) = target_selector(spell.shape, &nearest, &area) {
                plan.commands.push(format!(
                    "execute at {origin} run damage {selector} {damage:.1} {damage_type}"
                ));
            }
            if spell.effect == Effect::Drain {
                plan.commands.push(format!(
                    "effect give {origin} minecraft:instant_health 1 {amplifier}"
                ));
            }
        }
        Effect::Heal => {
            let selector = match spell.shape.targeting() {
                Targeting::Area => format!("@a[distance=..{radius:.1}]"),
                _ => origin.to_owned(),
            };
            plan.commands.push(format!(
                "execute at {origin} run effect give {selector} minecraft:instant_health 1 {amplifier}"
            ));
        }
        Effect::Slow | Effect::Root => {
            if let Some(selector) = target_selector(spell.shape, &nearest, &area) {
                let level = if spell.effect == Effect::Root { 6 } else { 1 };
                plan.commands.push(format!(
                    "execute at {origin} run effect give {selector} minecraft:slowness {seconds} {level}"
                ));
            }
        }
        Effect::Shield => {
            plan.commands.push(format!(
                "effect give {origin} minecraft:absorption {seconds} {amplifier}"
            ));
            plan.commands.push(format!(
                "effect give {origin} minecraft:resistance {seconds} 1"
            ));
        }
        Effect::Teleport => {
            plan.commands.push(format!(
                "execute at {origin} anchored eyes positioned ^ ^ ^{reach:.1} run tp {origin} ~ ~ ~"
            ));
        }
    }
    add_visuals(&mut plan, spell, origin, reach, radius, quality);
    plan.feedback = format!(
        "{} · {} · {:.1} power",
        spell.name,
        spell.element.name(),
        power
    );
    Ok(plan)
}

/// The selector a targeted effect lands on, when its shape can reach one.
fn target_selector(shape: Shape, nearest: &str, area: &str) -> Option<String> {
    match shape.targeting() {
        Targeting::NearestEnemy => Some(nearest.to_owned()),
        Targeting::Area => Some(area.to_owned()),
        Targeting::SelfOnly | Targeting::Displacement => None,
    }
}

/// Every cast carries its school's particles and sound, whatever it does.
fn add_visuals(
    plan: &mut EffectPlan,
    spell: &Spell,
    origin: &str,
    reach: f32,
    radius: f32,
    quality: RenderQuality,
) {
    let particle = spell.element.particle();
    let count = quality.particle_count();
    match spell.shape {
        Shape::Projectile | Shape::Beam => {
            let distance = reach * 0.6;
            plan.commands.push(format!(
                "execute at {origin} anchored eyes positioned ^ ^ ^{distance:.1} run particle {particle} ~ ~ ~ 0.4 0.4 0.4 0.02 {count}"
            ));
        }
        Shape::Zone | Shape::Nova | Shape::Wall => {
            plan.commands.push(format!(
                "execute at {origin} run particle {particle} ~ ~1 ~ {radius:.1} 1.0 {radius:.1} 0.02 {count}"
            ));
        }
        Shape::SelfCast => {
            plan.commands.push(format!(
                "execute at {origin} run particle {particle} ~ ~1 ~ 0.5 0.8 0.5 0.01 {count}"
            ));
        }
        Shape::Blink => {
            plan.commands.push(format!(
                "execute at {origin} run particle {particle} ~ ~1 ~ 0.4 0.9 0.4 0.02 {count}"
            ));
            plan.commands.push(format!(
                "execute at {origin} anchored eyes positioned ^ ^ ^{reach:.1} run particle {particle} ~ ~ ~ 0.4 0.9 0.4 0.02 {count}"
            ));
        }
    }
    plan.commands.push(format!(
        "execute at {origin} run playsound {} master @a[distance=..24] ~ ~ ~ 1.0 1.0",
        spell.element.sound()
    ));
}

/// Whether a name can go into an entity selector.
pub fn valid_origin(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 16
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balance::Limits;
    use crate::elements::Element;
    use crate::modifiers::Modifier;
    use crate::spell_builder::SpellBuilder;

    fn fireball() -> Spell {
        let limits = Limits::default();
        SpellBuilder::new(
            &limits,
            "pyro_fireball",
            "Fireball",
            Element::Pyro,
            Shape::Projectile,
            Effect::Damage,
        )
        .modifier(Modifier::Explosive)
        .build()
        .expect("balanced")
    }

    #[test]
    fn a_projectile_damages_the_nearest_enemy_then_draws_itself() {
        let plan = resolve(&fireball(), "Steve", 22.3, RenderQuality::Medium).expect("valid");
        assert_eq!(
            plan.commands.first().map(String::as_str),
            Some("execute at Steve run damage @e[distance=..24.0,sort=nearest,limit=1,type=!minecraft:player] 12.3 minecraft:on_fire")
        );
        assert!(plan
            .commands
            .iter()
            .any(|c| c.contains("particle minecraft:flame")));
        assert!(plan
            .commands
            .iter()
            .any(|c| c.contains("playsound minecraft:entity.blaze.shoot")));
        assert_eq!(plan.commands.len(), 3, "damage, particles, sound");
    }

    #[test]
    fn quality_changes_only_the_particle_count() {
        let low = resolve(&fireball(), "Steve", 22.3, RenderQuality::Low).expect("valid");
        let high = resolve(&fireball(), "Steve", 22.3, RenderQuality::High).expect("valid");
        assert!(low.commands[1].ends_with(" 4"));
        assert!(high.commands[1].ends_with(" 16"));
        assert_eq!(low.commands[0], high.commands[0]);
    }

    #[test]
    fn an_unusable_name_is_refused() {
        let spell = fireball();
        assert_eq!(
            resolve(&spell, "Steve; kill @e", 10.0, RenderQuality::Low),
            Err(CastError::InvalidOrigin)
        );
        assert_eq!(
            resolve(&spell, "", 10.0, RenderQuality::Low),
            Err(CastError::InvalidOrigin)
        );
    }

    #[test]
    fn a_blink_moves_the_caster_and_marks_both_ends() {
        let limits = Limits::default();
        let blink = SpellBuilder::new(
            &limits,
            "shadow_step",
            "Shadow Step",
            Element::Shadow,
            Shape::Blink,
            Effect::Teleport,
        )
        .build()
        .expect("balanced");
        let plan = resolve(&blink, "Alex", 0.0, RenderQuality::Medium).expect("valid");
        assert!(plan.commands[0]
            .starts_with("execute at Alex anchored eyes positioned ^ ^ ^12.0 run tp Alex ~ ~ ~"));
        let particles = plan
            .commands
            .iter()
            .filter(|c| c.contains("particle"))
            .count();
        assert_eq!(particles, 2);
    }

    #[test]
    fn a_zone_heals_everyone_around() {
        let limits = Limits::default();
        let sanctuary = SpellBuilder::new(
            &limits,
            "nature_sanctuary",
            "Healing Sanctuary",
            Element::Nature,
            Shape::Zone,
            Effect::Heal,
        )
        .build()
        .expect("balanced");
        let plan = resolve(&sanctuary, "Alex", 7.0, RenderQuality::Low).expect("valid");
        assert!(plan.commands[0].starts_with(
            "execute at Alex run effect give @a[distance=..6.0] minecraft:instant_health"
        ));
    }

    #[test]
    fn feedback_names_the_spell_and_its_school() {
        let plan = resolve(&fireball(), "Steve", 22.3, RenderQuality::Low).expect("valid");
        assert!(plan.feedback.contains("Fireball"));
        assert!(plan.feedback.contains("Pyromancy"));
    }
}
