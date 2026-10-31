//! The economy: every spell the mod ships with, and every spell the builder
//! will accept, stays inside the caps.

use arcanum_engine::balance::MIN_COST_PER_POWER;
use arcanum_engine::{
    Effect, Element, Limits, Modifier, Shape, Spell, SpellBuilder, SpellRegistry,
};

fn registry() -> SpellRegistry {
    SpellRegistry::builtin().expect("the spellbook builds")
}

#[test]
fn every_builtin_spell_respects_the_limits() {
    let limits = Limits::default();
    for spell in registry().all() {
        assert!(
            spell.cost <= limits.max_cost,
            "{} costs {}",
            spell.id,
            spell.cost
        );
        assert!(
            spell.power <= limits.max_power,
            "{} has power {}",
            spell.id,
            spell.power
        );
        assert!(
            spell.range <= limits.max_range,
            "{} reaches {}",
            spell.id,
            spell.range
        );
        assert!(
            spell.cooldown >= limits.min_cooldown && spell.cooldown <= limits.max_cooldown,
            "{} cools down in {}",
            spell.id,
            spell.cooldown
        );
        assert!(
            spell.cost >= spell.power * MIN_COST_PER_POWER,
            "{} is too efficient: {} mana for {} power",
            spell.id,
            spell.cost,
            spell.power
        );
    }
}

#[test]
fn every_school_ships_five_spells_and_the_book_is_thirty_strong() {
    let registry = registry();
    assert_eq!(registry.len(), 30);
    for element in Element::ALL {
        assert_eq!(registry.by_school(element).count(), 5, "{}", element.id());
    }
}

#[test]
fn nothing_requires_a_level_above_the_cap() {
    for spell in registry().all() {
        assert!(spell.required_level <= arcanum_engine::MAX_LEVEL);
    }
}

#[test]
fn the_strongest_spell_is_nowhere_near_one_shot_damage() {
    let registry = registry();
    let strongest = registry
        .all()
        .iter()
        .max_by(|a, b| a.damage().total_cmp(&b.damage()))
        .expect("a non-empty book");
    // 60 power is the ceiling, and damage is 55% of power.
    assert!(
        strongest.damage() <= 60.0 * 0.55 + 0.05,
        "{} hits for {}",
        strongest.id,
        strongest.damage()
    );
}

#[test]
fn the_builder_never_returns_a_spell_past_a_cap() {
    let limits = Limits::default();
    let mut built = 0_usize;
    let mut refused = 0_usize;
    for element in Element::ALL {
        for shape in Shape::ALL {
            for effect in Effect::ALL {
                for first in Modifier::ALL {
                    for second in Modifier::ALL {
                        for third in Modifier::ALL {
                            let mut chosen = vec![first];
                            if second != first {
                                chosen.push(second);
                            }
                            if third != first && third != second {
                                chosen.push(third);
                            }
                            let result = SpellBuilder::new(
                                &limits, "combo", "Combo", element, shape, effect,
                            )
                            .modifiers(&chosen)
                            .build();
                            match result {
                                Ok(spell) => {
                                    built += 1;
                                    assert!(spell.cost <= limits.max_cost);
                                    assert!(spell.power <= limits.max_power);
                                    assert!(spell.range <= limits.max_range);
                                    assert!(spell.cost >= spell.power * MIN_COST_PER_POWER);
                                }
                                Err(_) => refused += 1,
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(built > 0, "some combinations are legal");
    assert!(refused > 0, "the caps have to bite somewhere");
}

#[test]
fn a_modifier_makes_a_spell_cost_more_or_hit_harder() {
    let limits = Limits::default();
    let plain = SpellBuilder::new(
        &limits,
        "plain",
        "Plain",
        Element::Pyro,
        Shape::Projectile,
        Effect::Damage,
    )
    .build()
    .expect("a legal spell");
    let empowered = SpellBuilder::new(
        &limits,
        "strong",
        "Strong",
        Element::Pyro,
        Shape::Projectile,
        Effect::Damage,
    )
    .modifier(Modifier::Empowered)
    .build()
    .expect("a legal spell");
    assert!(empowered.power > plain.power);
    assert!(empowered.cost > plain.cost);
    assert!(empowered.cooldown >= plain.cooldown);
}

#[test]
fn arcane_costs_more_than_nature_for_the_same_shape() {
    let limits = Limits::default();
    let nature = SpellBuilder::new(
        &limits,
        "n",
        "N",
        Element::Nature,
        Shape::Beam,
        Effect::Damage,
    )
    .build()
    .expect("a legal spell");
    let arcane = SpellBuilder::new(
        &limits,
        "a",
        "A",
        Element::Arcane,
        Shape::Beam,
        Effect::Damage,
    )
    .build()
    .expect("a legal spell");
    assert!(arcane.cost > nature.cost);
    assert!(arcane.power > nature.power);
}

#[test]
fn every_spell_has_a_positive_cost_and_a_name() {
    for spell in registry().all() {
        assert!(spell.cost >= 1.0, "{}", spell.id);
        assert!(!spell.name.trim().is_empty());
        assert!(spell.describe().contains(&spell.name));
    }
}

#[test]
fn a_tighter_ceiling_refuses_what_the_default_allows() {
    let default = Limits::default();
    let strict = Limits::default().with_max_cost(12.0);
    let build = |limits: &Limits| -> Result<Spell, _> {
        SpellBuilder::new(
            limits,
            "m",
            "M",
            Element::Arcane,
            Shape::Nova,
            Effect::Damage,
        )
        .build()
    };
    assert!(build(&default).is_ok());
    assert!(build(&strict).is_err());
}
