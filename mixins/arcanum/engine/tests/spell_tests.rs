//! The spell model: composition, refusals and what a spell says about itself.

use arcanum_engine::{
    BuildError, Effect, Element, Limits, Modifier, Shape, SpellBuilder, SpellRegistry,
};

fn build(
    element: Element,
    shape: Shape,
    effect: Effect,
    modifiers: &[Modifier],
) -> Result<arcanum_engine::Spell, BuildError> {
    let limits = Limits::default();
    SpellBuilder::new(&limits, "test", "Test", element, shape, effect)
        .modifiers(modifiers)
        .build()
}

#[test]
fn the_documented_examples_can_be_composed() {
    // Fire + projectile + explosive = an exploding fireball.
    let fireball = build(
        Element::Pyro,
        Shape::Projectile,
        Effect::Damage,
        &[Modifier::Explosive],
    )
    .expect("a legal spell");
    assert!(fireball.has(Modifier::Explosive));
    assert_eq!(fireball.element, Element::Pyro);
    assert_eq!(fireball.shape, Shape::Projectile);

    // Ice + zone + slow = a blizzard.
    let blizzard = build(
        Element::Cryo,
        Shape::Zone,
        Effect::Slow,
        &[Modifier::Extended],
    )
    .expect("a legal spell");
    assert_eq!(blizzard.effect, Effect::Slow);

    // Lightning + beam + chain = chain lightning.
    let chain = build(
        Element::Electro,
        Shape::Beam,
        Effect::Damage,
        &[Modifier::Chaining],
    )
    .expect("a legal spell");
    assert!(chain.has(Modifier::Chaining));

    // Nature + zone + heal = a healing sanctuary.
    let sanctuary = build(Element::Nature, Shape::Zone, Effect::Heal, &[]).expect("a legal spell");
    assert_eq!(sanctuary.effect, Effect::Heal);

    // Shadow + projectile + drain = a vampiric orb.
    let orb = build(
        Element::Shadow,
        Shape::Projectile,
        Effect::Drain,
        &[Modifier::Draining],
    )
    .expect("a legal spell");
    assert_eq!(orb.effect, Effect::Drain);
}

#[test]
fn a_shape_that_only_reaches_the_caster_cannot_slow() {
    let error = build(Element::Cryo, Shape::SelfCast, Effect::Slow, &[]);
    assert_eq!(error, Err(BuildError::IncompatiblePair("self", "slow")));
}

#[test]
fn healing_does_not_fire_at_an_enemy() {
    let error = build(Element::Nature, Shape::Projectile, Effect::Heal, &[]);
    assert_eq!(
        error,
        Err(BuildError::IncompatiblePair("projectile", "heal"))
    );
}

#[test]
fn a_wall_can_root_and_a_blink_can_teleport() {
    assert!(build(Element::Cryo, Shape::Wall, Effect::Root, &[]).is_ok());
    assert!(build(Element::Shadow, Shape::Blink, Effect::Teleport, &[]).is_ok());
    assert!(build(Element::Shadow, Shape::Blink, Effect::Damage, &[]).is_err());
}

#[test]
fn efficient_and_empowered_cannot_be_stacked() {
    let error = build(
        Element::Pyro,
        Shape::Projectile,
        Effect::Damage,
        &[Modifier::Efficient, Modifier::Empowered],
    );
    assert_eq!(
        error,
        Err(BuildError::ConflictingModifiers("efficient", "empowered"))
    );
}

#[test]
fn the_same_modifier_twice_is_refused() {
    let error = build(
        Element::Pyro,
        Shape::Projectile,
        Effect::Damage,
        &[Modifier::Piercing, Modifier::Piercing],
    );
    assert_eq!(error, Err(BuildError::DuplicateModifier("piercing")));
}

#[test]
fn modifiers_are_capped_per_spell() {
    let error = build(
        Element::Pyro,
        Shape::Nova,
        Effect::Damage,
        &[
            Modifier::Explosive,
            Modifier::Chaining,
            Modifier::Piercing,
            Modifier::Extended,
        ],
    );
    assert!(matches!(
        error,
        Err(BuildError::TooManyModifiers { count: 4, max: 3 })
    ));
}

#[test]
fn a_spell_describes_itself_for_the_grimoire() {
    let spell = build(
        Element::Pyro,
        Shape::Projectile,
        Effect::Damage,
        &[Modifier::Explosive],
    )
    .expect("a legal spell");
    let text = spell.describe();
    assert!(text.contains("Pyromancy"));
    assert!(text.contains("projectile"));
    assert!(text.contains("damage"));
    assert!(text.contains("Explosive"));
}

#[test]
fn a_cast_is_worth_experience() {
    let spell = build(Element::Arcane, Shape::Beam, Effect::Damage, &[]).expect("a legal spell");
    assert!(spell.xp_reward() > 0);
    assert!(spell.damage() > 0.0);
}

#[test]
fn every_school_and_shape_pairing_the_book_uses_is_legal() {
    let registry = SpellRegistry::builtin().expect("the spellbook builds");
    for spell in registry.all() {
        assert!(!spell.id.is_empty());
        assert!(!spell.name.is_empty());
        assert!(spell.cost > 0.0);
        assert!(spell.power >= 0.0);
        assert!(spell.required_level >= 1 && spell.required_level <= arcanum_engine::MAX_LEVEL);
    }
}
