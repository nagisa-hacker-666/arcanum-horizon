//! Prints Arcanum's spellbook as a Markdown table, so `docs/SPELLS.md` is
//! never out of date by hand.
//!
//! `cargo run -p arcanum-engine --example spellbook`

use arcanum_engine::{Element, SpellRegistry};

fn main() {
    let registry = SpellRegistry::builtin().expect("the spellbook builds");
    for element in Element::ALL {
        println!("### {}", element.name());
        println!();
        println!(
            "| Spell | Shape | Effect | Modifiers | Level | Mana | Power | Range | Cooldown |"
        );
        println!("| --- | --- | --- | --- | --- | --- | --- | --- | --- |");
        for spell in registry.by_school(element) {
            let modifiers = if spell.modifiers.is_empty() {
                "—".to_owned()
            } else {
                spell
                    .modifiers
                    .iter()
                    .map(|modifier| modifier.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            println!(
                "| {} (`{}`) | {} | {} | {} | {} | {:.1} | {:.1} | {:.0} | {:.1}s |",
                spell.name,
                spell.id,
                spell.shape.name(),
                spell.effect.name(),
                modifiers,
                spell.required_level,
                spell.cost,
                spell.power,
                spell.range,
                spell.cooldown as f32 / 20.0
            );
        }
        println!();
    }
}
