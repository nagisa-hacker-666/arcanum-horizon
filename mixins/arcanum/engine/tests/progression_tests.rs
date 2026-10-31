//! Levels, stats, mastery, talents and specializations.

use arcanum_engine::progression::Stat;
use arcanum_engine::{Element, Progression, MAX_LEVEL, TALENTS};

#[test]
fn the_curve_never_dips_and_ends_at_the_cap() {
    let mut previous = 0;
    for level in 1..MAX_LEVEL {
        let need = Progression::xp_for_next(level);
        assert!(need > previous, "level {level} needs {need}");
        previous = need;
    }
    assert_eq!(Progression::xp_for_next(MAX_LEVEL), 0);
}

#[test]
fn a_mage_cannot_rise_past_level_fifty() {
    let mut progression = Progression::new();
    progression.add_xp(u64::MAX / 2);
    assert_eq!(progression.level, MAX_LEVEL);
    let before = progression.xp;
    assert_eq!(progression.add_xp(1_000_000), 0);
    assert_eq!(progression.xp, before, "experience past the cap is ignored");
}

#[test]
fn levelling_pays_two_points_a_level() {
    let mut progression = Progression::new();
    let gained = progression.add_xp(Progression::total_xp_for(6));
    assert_eq!(gained, 5);
    assert_eq!(
        progression.points,
        5 * arcanum_engine::balance::STAT_POINTS_PER_LEVEL
    );
}

#[test]
fn points_go_where_the_player_puts_them() {
    let mut progression = Progression::new();
    progression.add_xp(Progression::total_xp_for(3));
    let points = progression.points;
    for _ in 0..points {
        progression.spend_point(Stat::Power).expect("a point");
    }
    assert_eq!(progression.stats.power, points as u16);
    assert!(progression.spend_point(Stat::Power).is_err());
    assert_eq!(
        progression.stats.total(),
        points as u16,
        "no point appears from nowhere"
    );
}

#[test]
fn every_stat_changes_something_the_mod_controls() {
    let mut progression = Progression::new();
    let base_power = progression.power_multiplier(Element::Pyro);
    let base_cost = progression.cost_multiplier(Element::Pyro);
    let base_cooldown = progression.cooldown_multiplier();
    let base_mana = progression.max_mana();
    let base_regen = progression.regen_multiplier();
    progression.stats.add(Stat::Power);
    progression.stats.add(Stat::Mastery);
    progression.stats.add(Stat::Precision);
    progression.stats.add(Stat::Vitality);
    progression.stats.add(Stat::Concentration);
    assert!(progression.power_multiplier(Element::Pyro) > base_power);
    assert!(progression.cost_multiplier(Element::Pyro) < base_cost);
    assert!(progression.cooldown_multiplier() < base_cooldown);
    assert!(progression.max_mana() > base_mana);
    assert!(progression.regen_multiplier() > base_regen);
}

#[test]
fn mastery_is_earned_per_school_and_stops_at_a_hundred() {
    let mut progression = Progression::new();
    progression.add_mastery(Element::Shadow, 25.0);
    assert_eq!(progression.mastery(Element::Shadow), 25.0);
    assert_eq!(
        progression.mastery(Element::Pyro),
        0.0,
        "schools are separate"
    );
    for _ in 0..100 {
        progression.add_mastery(Element::Shadow, 10.0);
    }
    assert_eq!(progression.mastery(Element::Shadow), 100.0);
}

#[test]
fn a_specialization_needs_level_ten_and_is_permanent() {
    let mut progression = Progression::new();
    assert!(progression.choose_specialization(Element::Cryo).is_err());
    progression.add_xp(Progression::total_xp_for(10));
    progression
        .choose_specialization(Element::Cryo)
        .expect("level 10");
    assert!(progression.choose_specialization(Element::Pyro).is_err());
    assert_eq!(progression.specialization, Some(Element::Cryo));
    assert!(
        progression.power_multiplier(Element::Cryo) > progression.power_multiplier(Element::Pyro),
        "the specialization only helps its own school"
    );
}

#[test]
fn every_talent_is_reachable_and_gated() {
    for talent in TALENTS {
        let mut progression = Progression::new();
        progression.add_xp(Progression::total_xp_for(talent.level));
        if talent.specialization {
            assert!(
                progression.unlock_talent(talent.id).is_err(),
                "{} needs its specialization",
                talent.id
            );
            progression
                .choose_specialization(talent.element)
                .expect("level is high enough");
        }
        progression
            .unlock_talent(talent.id)
            .unwrap_or_else(|e| panic!("{}: {e}", talent.id));
        assert!(progression.talents().contains(&talent.id));
    }
}

#[test]
fn talents_of_one_school_do_not_help_another() {
    let mut progression = Progression::new();
    progression.add_xp(Progression::total_xp_for(30));
    progression
        .choose_specialization(Element::Pyro)
        .expect("level 30");
    progression.unlock_talent("pyro_ember").expect("level 5");
    let pyro = progression.power_multiplier(Element::Pyro);
    let cryo = progression.power_multiplier(Element::Cryo);
    assert!(pyro > cryo);
}

#[test]
fn two_mages_can_grow_in_different_directions() {
    let mut bruiser = Progression::new();
    let mut scholar = Progression::new();
    bruiser.add_xp(Progression::total_xp_for(20));
    scholar.add_xp(Progression::total_xp_for(20));
    for _ in 0..bruiser.points {
        bruiser.spend_point(Stat::Power).ok();
    }
    for _ in 0..scholar.points {
        scholar.spend_point(Stat::Mastery).ok();
    }
    assert!(bruiser.power_multiplier(Element::Pyro) > scholar.power_multiplier(Element::Pyro));
    assert!(scholar.cost_multiplier(Element::Pyro) < bruiser.cost_multiplier(Element::Pyro));
}
