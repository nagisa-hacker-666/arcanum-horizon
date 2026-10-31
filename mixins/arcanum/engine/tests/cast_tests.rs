//! What a cast turns into, and the client/server message codecs.

use arcanum_engine::net::{ClientMessage, StateSnapshot};
use arcanum_engine::{
    codec, resolve, CastError, Effect, Element, Limits, Modifier, PlayerState, Progression,
    RenderQuality, Shape, SpellBuilder, SpellRegistry,
};

fn registry() -> SpellRegistry {
    SpellRegistry::builtin().expect("the spellbook builds")
}

fn mage(name: &str) -> PlayerState {
    let mut player = PlayerState::new(11, name);
    player.progression.add_xp(Progression::total_xp_for(30));
    player.refresh();
    for spell in registry().all() {
        if spell.required_level <= 30 {
            player.learn(spell).expect("learnable");
        }
    }
    player
}

#[test]
fn a_cast_is_a_list_of_server_commands() {
    let mut player = mage("Alex");
    let spell = registry().get("pyro_fireball").expect("known").clone();
    let plan = player.cast(&spell, RenderQuality::Medium).expect("cast");
    assert_eq!(plan.commands.len(), 3);
    assert!(plan.commands[0].starts_with("execute at Alex run damage @e["));
    assert!(plan.commands[0].ends_with(" minecraft:on_fire"));
    assert!(plan
        .commands
        .iter()
        .any(|c| c.contains("particle minecraft:flame")));
    assert!(plan.commands.iter().any(|c| c.contains("playsound")));
    assert!(plan.feedback.contains("Fireball"));
}

#[test]
fn a_self_shield_protects_the_caster() {
    let mut player = mage("Alex");
    let spell = registry().get("arcane_shield").expect("known").clone();
    let plan = player.cast(&spell, RenderQuality::Low).expect("cast");
    assert!(plan.commands[0].starts_with("effect give Alex minecraft:absorption"));
    assert!(plan.commands[1].starts_with("effect give Alex minecraft:resistance"));
}

#[test]
fn a_shadow_step_teleports_and_leaves_particles_at_both_ends() {
    let mut player = mage("Alex");
    let spell = registry().get("shadow_step").expect("known").clone();
    let plan = player.cast(&spell, RenderQuality::High).expect("cast");
    assert!(plan.commands[0].contains("run tp Alex ~ ~ ~"));
    assert_eq!(
        plan.commands
            .iter()
            .filter(|c| c.contains("particle"))
            .count(),
        2
    );
}

#[test]
fn a_drain_hurts_a_target_and_heals_the_caster() {
    let mut player = mage("Alex");
    let spell = registry().get("shadow_drain").expect("known").clone();
    let plan = player.cast(&spell, RenderQuality::Low).expect("cast");
    assert!(plan.commands[0].contains("run damage"));
    assert!(plan.commands[1].starts_with("effect give Alex minecraft:instant_health"));
}

#[test]
fn a_root_holds_what_it_reaches() {
    let mut player = mage("Alex");
    let spell = registry().get("nature_roots").expect("known").clone();
    let plan = player.cast(&spell, RenderQuality::Low).expect("cast");
    assert!(plan.commands[0].contains("minecraft:slowness"));
    assert!(plan.commands[0].trim_end().ends_with(" 6 6") || plan.commands[0].contains(" 6"));
}

#[test]
fn the_same_cast_is_the_same_plan_every_time() {
    let first = {
        let mut player = mage("Alex");
        let spell = registry().get("electro_chain").expect("known").clone();
        player.cast(&spell, RenderQuality::Medium).expect("cast")
    };
    let second = {
        let mut player = mage("Alex");
        let spell = registry().get("electro_chain").expect("known").clone();
        player.cast(&spell, RenderQuality::Medium).expect("cast")
    };
    assert_eq!(first, second, "the plan carries no client state");
}

#[test]
fn a_caster_with_no_learned_spell_gets_an_empty_selection() {
    let mut player = PlayerState::new(1, "Novice");
    assert_eq!(
        player.try_cast(&registry(), RenderQuality::Low),
        Err(CastError::EmptySelection)
    );
}

#[test]
fn a_refused_cast_keeps_mana_and_cooldown_untouched() {
    let mut player = mage("Alex");
    let spell = registry().get("arcane_singularity").expect("known").clone();
    player.mana.empty();
    let before = player.cooldowns.remaining(&spell.id);
    assert!(player.cast(&spell, RenderQuality::Low).is_err());
    assert_eq!(player.mana.current(), 0.0);
    assert_eq!(player.cooldowns.remaining(&spell.id), before);
}

#[test]
fn a_custom_spell_reaches_the_server_like_a_builtin_one() {
    let limits = Limits::default();
    let spell = SpellBuilder::new(
        &limits,
        "custom_glacier",
        "Glacier",
        Element::Cryo,
        Shape::Zone,
        Effect::Slow,
    )
    .modifier(Modifier::Extended)
    .build()
    .expect("a legal spell");
    let plan = resolve(&spell, "Alex", spell.power, RenderQuality::Low).expect("valid");
    assert!(plan.commands[0].contains("minecraft:slowness"));
    assert!(plan.commands[0].contains("minecraft:slowness"));
}

#[test]
fn a_save_survives_a_round_trip_through_text() {
    let mut player = mage("Alex");
    let spell = registry().get("pyro_fireball").expect("known").clone();
    player.cast(&spell, RenderQuality::Low).expect("cast");
    let text = codec::encode(&player);
    let back = codec::decode(&text).expect("readable");
    assert_eq!(back.progression.level, player.progression.level);
    assert_eq!(back.known(), player.known());
    assert_eq!(back.name, player.name);
}

#[test]
fn client_requests_round_trip_and_junk_is_dropped() {
    assert_eq!(ClientMessage::decode("cast"), Some(ClientMessage::Cast));
    assert_eq!(
        ClientMessage::decode("select -1"),
        Some(ClientMessage::Select(-1))
    );
    assert_eq!(ClientMessage::decode("mana 100"), None);
}

#[test]
fn a_state_snapshot_describes_the_hud() {
    let mut player = mage("Alex");
    let spell = registry().get("pyro_fireball").expect("known").clone();
    player.cast(&spell, RenderQuality::Low).expect("cast");
    let snapshot = StateSnapshot {
        level: player.progression.level,
        xp_into: player.progression.level_progress().0,
        xp_needed: player.progression.level_progress().1,
        mana: player.mana.current(),
        mana_max: player.mana.max(),
        resonance: player.resonance.value(),
        cooldown: player.cooldowns.remaining(&spell.id),
        spell: spell.name.clone(),
        element: spell.element.id().to_owned(),
    };
    let decoded = StateSnapshot::decode(&snapshot.encode()).expect("round trip");
    // The wire format carries one decimal: what the bar draws, no more.
    assert_eq!(decoded.level, snapshot.level);
    assert_eq!(decoded.xp_into, snapshot.xp_into);
    assert_eq!(decoded.xp_needed, snapshot.xp_needed);
    assert_eq!(decoded.cooldown, snapshot.cooldown);
    assert_eq!(decoded.spell, snapshot.spell);
    assert_eq!(decoded.element, snapshot.element);
    assert!((decoded.mana - snapshot.mana).abs() < 0.1);
    assert!((decoded.mana_max - snapshot.mana_max).abs() < 0.1);
    assert!((decoded.resonance - snapshot.resonance).abs() < 0.1);
    assert!(decoded.mana_ratio() > 0.0 && decoded.mana_ratio() <= 1.0);
    assert!(!decoded.is_ready());
}
