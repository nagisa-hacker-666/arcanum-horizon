//! Mana, regeneration and resonance, through the public API.

use arcanum_engine::{
    Effect, Element, Limits, Mana, PlayerState, Progression, Resonance, Shape, SpellBuilder,
    SpellRegistry,
};

fn registry() -> SpellRegistry {
    SpellRegistry::builtin().expect("the spellbook builds")
}

fn mage() -> PlayerState {
    let mut player = PlayerState::new(7, "Steve");
    player.progression.add_xp(Progression::total_xp_for(20));
    player.refresh();
    for spell in registry().all() {
        if spell.required_level <= 20 {
            player.learn(spell).expect("learnable at level 20");
        }
    }
    player
}

#[test]
fn a_pool_regenerates_over_time_and_stops_at_the_top() {
    let mut mana = Mana::new(100.0);
    mana.spend(60.0);
    assert_eq!(mana.current(), 40.0);
    mana.tick(10);
    let after = mana.current();
    assert!(after > 40.0, "regeneration moved the pool");
    mana.tick(100_000);
    assert_eq!(mana.current(), mana.max());
    assert_eq!(mana.ratio(), 1.0);
}

#[test]
fn a_pool_never_goes_negative() {
    let mut mana = Mana::new(20.0);
    assert!(mana.spend(20.0));
    assert!(!mana.spend(1.0));
    assert_eq!(mana.current(), 0.0);
    assert!(mana.is_empty());
}

#[test]
fn the_pool_grows_with_level_and_vitality() {
    let low = PlayerState::new(1, "Novice");
    let mut high = mage();
    for _ in 0..10 {
        high.progression
            .add_xp(Progression::xp_for_next(high.progression.level));
        high.refresh();
    }
    assert!(high.mana.max() > low.mana.max(), "levels widen the pool");
    let before = high.mana.max();
    high.progression
        .spend_point(arcanum_engine::progression::Stat::Vitality)
        .expect("a point from the last level");
    high.refresh();
    assert!(high.mana.max() >= before);
}

#[test]
fn regeneration_follows_concentration() {
    let mut player = mage();
    player.mana.spend(player.mana.max());
    let slow = {
        player.tick(20);
        player.mana.current()
    };
    let mut focused = mage();
    focused.mana.spend(focused.mana.max());
    for _ in 0..20 {
        focused
            .progression
            .spend_point(arcanum_engine::progression::Stat::Concentration)
            .ok();
    }
    focused.refresh();
    focused.tick(20);
    assert!(
        focused.mana.current() > slow,
        "concentration must regenerate faster"
    );
}

#[test]
fn resonance_charges_with_casts_and_opens_an_ultimate() {
    let mut player = mage();
    let spell = registry().get("pyro_fireball").expect("known").clone();
    assert!(!player.resonance.is_ready());
    for _ in 0..40 {
        player.cooldowns.clear();
        player.mana.restore(player.mana.max());
        player
            .cast(&spell, arcanum_engine::RenderQuality::Low)
            .expect("cast");
    }
    assert!(player.resonance.value() > 0.0, "casting charges resonance");
    assert!(player.resonance.is_ready(), "40 casts charge an ultimate");
    assert!(player.resonance.spend_ultimate());
    assert!(!player.resonance.is_ready(), "spending it empties it");
    assert_eq!(player.resonance.value(), 0.0);
}

#[test]
fn a_quiet_mage_loses_resonance() {
    let mut resonance = Resonance::default();
    resonance.gain(50.0);
    let charged = resonance.value();
    for _ in 0..500 {
        resonance.tick();
    }
    assert!(resonance.value() < charged);
    assert!(resonance.value() >= 0.0);
}

#[test]
fn the_three_resources_move_together_on_a_cast() {
    let mut player = mage();
    let spell = registry().get("cryo_ice_bolt").expect("known").clone();
    let mana = player.mana.current();
    player
        .cast(&spell, arcanum_engine::RenderQuality::Medium)
        .expect("cast");
    assert!(player.mana.current() < mana, "mana is spent");
    assert!(
        player.cooldowns.remaining(&spell.id) > 0,
        "the cooldown starts"
    );
    assert!(player.resonance.value() > 0.0, "resonance charges");
}

#[test]
fn every_school_has_a_first_spell_a_level_one_mage_can_learn() {
    let registry = registry();
    for element in Element::ALL {
        let first = registry
            .by_school(element)
            .find(|spell| spell.required_level <= 1)
            .unwrap_or_else(|| panic!("{} has no level 1 spell", element.id()));
        let mut player = PlayerState::new(1, "Novice");
        player.learn(first).expect("learnable at level 1");
    }
}

#[test]
fn a_cheap_custom_spell_still_costs_mana() {
    let limits = Limits::default();
    let spell = SpellBuilder::new(
        &limits,
        "custom_ward",
        "Ward",
        Element::Nature,
        Shape::SelfCast,
        Effect::Shield,
    )
    .build()
    .expect("a legal spell");
    assert!(spell.cost >= 1.0);
    assert!(spell.cost >= spell.power * arcanum_engine::balance::MIN_COST_PER_POWER);
}

#[test]
fn the_efficiency_floor_refuses_a_free_high_power_spell() {
    let limits = Limits::default();
    let refused = SpellBuilder::new(
        &limits,
        "free_spark",
        "Free Spark",
        Element::Electro,
        Shape::Projectile,
        Effect::Damage,
    )
    .modifier(arcanum_engine::Modifier::Efficient)
    .build();
    assert!(
        matches!(
            refused,
            Err(arcanum_engine::BuildError::TooEfficient { .. })
        ),
        "a spell cheaper than 90% of its own power must be refused, got {refused:?}"
    );
}
