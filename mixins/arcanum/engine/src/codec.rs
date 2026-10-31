//! How a mage's progression is saved.
//!
//! One text file per player under `config/arcanum/players/`, in a format that
//! is diffable, forward compatible (unknown keys are ignored) and versioned
//! (an unknown version is refused rather than half-read). No dependency is
//! pulled in for it: a `serde` crate would have to exist in Horizon's
//! `Cargo.lock`, and the format is small enough to write by hand.

use std::fmt;

use crate::elements::Element;
use crate::mana::Mana;
use crate::player::PlayerState;
use crate::progression::{self, Stats};

const VERSION: &str = "arcanum-state 1";

/// Why a save could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodecError {
    Empty,
    UnsupportedVersion,
    Missing(&'static str),
    Bad(&'static str),
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("the save is empty"),
            Self::UnsupportedVersion => f.write_str("the save was written by another version"),
            Self::Missing(key) => write!(f, "the save has no `{key}` line"),
            Self::Bad(key) => write!(f, "the save's `{key}` line is not readable"),
        }
    }
}

impl std::error::Error for CodecError {}

/// Writes a mage to text.
pub fn encode(player: &PlayerState) -> String {
    let progression = &player.progression;
    let mastery: Vec<String> = Element::ALL
        .iter()
        .map(|element| format!("{:.2}", progression.mastery(*element)))
        .collect();
    let mut out = String::new();
    out.push_str(VERSION);
    out.push('\n');
    out.push_str(&format!("uuid {}\n", player.uuid));
    out.push_str(&format!("name {}\n", player.name));
    out.push_str(&format!("xp {}\n", progression.xp));
    out.push_str(&format!("level {}\n", progression.level));
    out.push_str(&format!("points {}\n", progression.points));
    out.push_str(&format!(
        "stats {} {} {} {} {}\n",
        progression.stats.power,
        progression.stats.mastery,
        progression.stats.precision,
        progression.stats.vitality,
        progression.stats.concentration
    ));
    out.push_str(&format!("mastery {}\n", mastery.join(" ")));
    out.push_str(&format!(
        "spec {}\n",
        progression
            .specialization
            .map(Element::id)
            .unwrap_or("none")
    ));
    out.push_str(&format!("talents {}\n", progression.talents().join(" ")));
    out.push_str(&format!("known {}\n", player.known().join(" ")));
    out.push_str(&format!("selected {}\n", player.selected_index()));
    out.push_str(&format!(
        "mana {:.2} {:.2}\n",
        player.mana.current(),
        player.mana.max()
    ));
    out
}

/// Reads a mage back.
pub fn decode(text: &str) -> Result<PlayerState, CodecError> {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header = lines.next().ok_or(CodecError::Empty)?;
    if header.trim() != VERSION {
        return Err(CodecError::UnsupportedVersion);
    }

    let mut uuid = None;
    let mut name = None;
    let mut xp = None;
    let mut level = None;
    let mut points = None;
    let mut stats = None;
    let mut mastery = None;
    let mut spec = None;
    let mut talents: Vec<&'static str> = Vec::new();
    let mut known: Vec<String> = Vec::new();
    let mut selected = 0;
    let mut mana = None;

    for line in lines {
        let (key, value) = line.split_once(' ').unwrap_or((line.trim(), ""));
        match key {
            "uuid" => {
                uuid = Some(
                    value
                        .trim()
                        .parse::<u128>()
                        .map_err(|_| CodecError::Bad("uuid"))?,
                )
            }
            "name" => name = Some(value.trim().to_owned()),
            "xp" => {
                xp = Some(
                    value
                        .trim()
                        .parse::<u64>()
                        .map_err(|_| CodecError::Bad("xp"))?,
                )
            }
            "level" => {
                level = Some(
                    value
                        .trim()
                        .parse::<u32>()
                        .map_err(|_| CodecError::Bad("level"))?,
                )
            }
            "points" => {
                points = Some(
                    value
                        .trim()
                        .parse::<u32>()
                        .map_err(|_| CodecError::Bad("points"))?,
                )
            }
            "stats" => {
                let parsed: Result<Vec<u16>, _> =
                    value.split_whitespace().map(str::parse::<u16>).collect();
                let parsed = parsed.map_err(|_| CodecError::Bad("stats"))?;
                let [power, mastery_stat, precision, vitality, concentration] = parsed.as_slice()
                else {
                    return Err(CodecError::Bad("stats"));
                };
                stats = Some(Stats {
                    power: *power,
                    mastery: *mastery_stat,
                    precision: *precision,
                    vitality: *vitality,
                    concentration: *concentration,
                });
            }
            "mastery" => {
                let parsed: Result<Vec<f32>, _> =
                    value.split_whitespace().map(str::parse::<f32>).collect();
                let parsed = parsed.map_err(|_| CodecError::Bad("mastery"))?;
                if parsed.len() != Element::ALL.len() {
                    return Err(CodecError::Bad("mastery"));
                }
                let mut array = [0.0f32; 6];
                for (slot, value) in array.iter_mut().zip(parsed) {
                    *slot = value.clamp(0.0, 100.0);
                }
                mastery = Some(array);
            }
            "spec" => {
                let value = value.trim();
                spec = if value == "none" {
                    Some(None)
                } else {
                    Some(Some(
                        Element::from_id(value).ok_or(CodecError::Bad("spec"))?,
                    ))
                };
            }
            "talents" => {
                for id in value.split_whitespace() {
                    let talent = progression::talent(id).ok_or(CodecError::Bad("talents"))?;
                    talents.push(talent.id);
                }
            }
            "known" => known.extend(value.split_whitespace().map(str::to_owned)),
            "selected" => {
                selected = value
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| CodecError::Bad("selected"))?
            }
            "mana" => {
                let parsed: Result<Vec<f32>, _> =
                    value.split_whitespace().map(str::parse::<f32>).collect();
                let parsed = parsed.map_err(|_| CodecError::Bad("mana"))?;
                let [current, max] = parsed.as_slice() else {
                    return Err(CodecError::Bad("mana"));
                };
                mana = Some((*current, *max));
            }
            // Unknown keys are a newer save: keep what we understand.
            _ => {}
        }
    }

    let uuid = uuid.ok_or(CodecError::Missing("uuid"))?;
    let name = name.ok_or(CodecError::Missing("name"))?;
    let mut player = PlayerState::new(uuid, name);
    player.progression.restore(
        xp.ok_or(CodecError::Missing("xp"))?,
        level.ok_or(CodecError::Missing("level"))?,
        points.ok_or(CodecError::Missing("points"))?,
        stats.ok_or(CodecError::Missing("stats"))?,
        mastery.ok_or(CodecError::Missing("mastery"))?,
        spec.ok_or(CodecError::Missing("spec"))?,
    );
    for talent in talents {
        player.progression.restore_talent(talent);
    }
    player.restore_known(known, selected);
    let (current, saved_max) = mana.ok_or(CodecError::Missing("mana"))?;
    player.mana = Mana::from_parts(current, saved_max);
    // The pool's size is derived from level and stats, not trusted from the
    // file: a hand-edited save cannot raise it.
    player.refresh();
    player.mana.restore(current);
    Ok(player)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balance;
    use crate::progression::{Progression, Stat};

    fn mage() -> PlayerState {
        let mut player = PlayerState::new(42, "Steve");
        player.progression.add_xp(Progression::total_xp_for(20));
        player.progression.spend_point(Stat::Power).expect("points");
        player.progression.add_mastery(Element::Pyro, 37.5);
        player
            .progression
            .choose_specialization(Element::Pyro)
            .expect("level 20");
        player
            .progression
            .unlock_talent("pyro_ember")
            .expect("level 20");
        player.restore_known(
            vec!["pyro_fireball".to_owned(), "cryo_ice_bolt".to_owned()],
            1,
        );
        player.refresh();
        player.mana.restore(player.mana.max());
        player
    }

    #[test]
    fn a_save_round_trips() {
        let player = mage();
        let text = encode(&player);
        let back = decode(&text).expect("readable");
        assert_eq!(back.uuid, player.uuid);
        assert_eq!(back.name, player.name);
        assert_eq!(back.progression.level, player.progression.level);
        assert_eq!(back.progression.xp, player.progression.xp);
        assert_eq!(back.progression.stats, player.progression.stats);
        assert_eq!(back.progression.specialization, Some(Element::Pyro));
        assert_eq!(back.progression.talents(), ["pyro_ember"]);
        assert_eq!(back.known(), player.known());
        assert_eq!(back.selected_index(), 1);
        assert!((back.progression.mastery(Element::Pyro) - 37.5).abs() < 0.01);
    }

    #[test]
    fn a_save_of_another_version_is_refused() {
        assert_eq!(
            decode("arcanum-state 2\n"),
            Err(CodecError::UnsupportedVersion)
        );
        assert_eq!(decode(""), Err(CodecError::Empty));
    }

    #[test]
    fn a_missing_line_says_which_one() {
        let text = "arcanum-state 1\nuuid 1\nname Steve\n";
        assert_eq!(decode(text), Err(CodecError::Missing("xp")));
    }

    #[test]
    fn a_broken_line_is_refused() {
        let text =
            "arcanum-state 1\nuuid 1\nname Steve\nxp 1\nlevel 2\npoints 0\nstats a b c d e\n";
        assert_eq!(decode(text), Err(CodecError::Bad("stats")));
    }

    #[test]
    fn mana_never_exceeds_the_pool_after_loading() {
        let text = "arcanum-state 1\nuuid 1\nname Steve\nxp 0\nlevel 1\npoints 0\nstats 0 0 0 0 0\nmastery 0 0 0 0 0 0\nspec none\ntalents \nknown \nselected 0\nmana 9999.0 50.0\n";
        let player = decode(text).expect("readable");
        assert!(player.mana.current() <= player.mana.max());
        assert_eq!(player.mana.max(), balance::BASE_MANA);
    }
}
