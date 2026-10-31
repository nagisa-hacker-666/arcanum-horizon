//! The bytes the two halves of Arcanum exchange (hook API 40).
//!
//! Both directions carry plain text, so a message can be read in a log and a
//! mismatched version is rejected instead of misread. The client only ever
//! asks; the server answers with a snapshot of numbers it owns.

use crate::effects::RenderQuality;

/// What a client can ask for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientMessage {
    /// Cast the spell the wheel has selected.
    Cast,
    /// Move the selection by this many steps (the mouse wheel).
    Select(i32),
    /// Send the state again, after a screen or a respawn.
    Sync,
}

impl ClientMessage {
    /// Encodes it: `cast`, `select -1`, `sync`.
    pub fn encode(&self) -> String {
        match self {
            Self::Cast => "cast".to_owned(),
            Self::Select(delta) => format!("select {delta}"),
            Self::Sync => "sync".to_owned(),
        }
    }

    /// Reads what the client sent, or nothing when it is not ours.
    pub fn decode(text: &str) -> Option<Self> {
        let text = text.trim();
        match text {
            "cast" => Some(Self::Cast),
            "sync" => Some(Self::Sync),
            _ => {
                let delta = text.strip_prefix("select ")?.trim().parse::<i32>().ok()?;
                if delta.unsigned_abs() > 100 {
                    return None;
                }
                Some(Self::Select(delta))
            }
        }
    }
}

/// The numbers the HUD draws. Every one of them comes from the server, which
/// is the only side allowed to change them.
#[derive(Clone, Debug, PartialEq)]
pub struct StateSnapshot {
    pub level: u32,
    pub xp_into: u64,
    pub xp_needed: u64,
    pub mana: f32,
    pub mana_max: f32,
    pub resonance: f32,
    pub cooldown: u32,
    /// The selected spell, or an empty string when the grimoire has none.
    pub spell: String,
    /// The selected spell's school id, or empty.
    pub element: String,
}

impl StateSnapshot {
    /// How full the mana bar is.
    pub fn mana_ratio(&self) -> f32 {
        if self.mana_max <= 0.0 {
            0.0
        } else {
            (self.mana / self.mana_max).clamp(0.0, 1.0)
        }
    }

    /// How full the experience bar is.
    pub fn xp_ratio(&self) -> f32 {
        if self.xp_needed == 0 {
            1.0
        } else {
            (self.xp_into as f32 / self.xp_needed as f32).clamp(0.0, 1.0)
        }
    }

    /// How full the resonance ring is, from 0 to 1.
    pub fn resonance_ratio(&self) -> f32 {
        (self.resonance / crate::balance::RESONANCE_MAX).clamp(0.0, 1.0)
    }

    pub fn is_ready(&self) -> bool {
        self.cooldown == 0
    }

    /// Encodes it as one line, `v1;key=value;...`.
    pub fn encode(&self) -> String {
        format!(
            "v1;level={};xp={}/{};mana={:.1}/{:.1};res={:.1};cd={};spell={};el={}",
            self.level,
            self.xp_into,
            self.xp_needed,
            self.mana,
            self.mana_max,
            self.resonance,
            self.cooldown,
            self.spell,
            self.element
        )
    }

    /// Reads a line [`encode`](Self::encode) wrote, or nothing.
    pub fn decode(text: &str) -> Option<Self> {
        let mut parts = text.trim().split(';');
        if parts.next()? != "v1" {
            return None;
        }
        let mut level = None;
        let mut xp = None;
        let mut mana = None;
        let mut res = None;
        let mut cd = None;
        let mut spell = None;
        let mut element = None;
        for part in parts {
            let (key, value) = part.split_once('=')?;
            match key {
                "level" => level = value.parse().ok(),
                "xp" => {
                    let (into, needed) = value.split_once('/')?;
                    xp = Some((into.parse().ok()?, needed.parse().ok()?));
                }
                "mana" => {
                    let (current, max) = value.split_once('/')?;
                    mana = Some((current.parse().ok()?, max.parse().ok()?));
                }
                "res" => res = value.parse().ok(),
                "cd" => cd = value.parse().ok(),
                "spell" => spell = Some(value.to_owned()),
                "el" => element = Some(value.to_owned()),
                // An unknown key is a newer server: ignore it, do not fail.
                _ => {}
            }
        }
        let (xp_into, xp_needed) = xp?;
        let (mana, mana_max) = mana?;
        Some(Self {
            level: level?,
            xp_into,
            xp_needed,
            mana,
            mana_max,
            resonance: res?,
            cooldown: cd?,
            spell: spell?,
            element: element?,
        })
    }
}

/// The quality a player picked, as the settings page stores it.
pub fn quality_from_setting(value: Option<&str>) -> RenderQuality {
    value
        .and_then(RenderQuality::from_id)
        .unwrap_or(RenderQuality::Medium)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_messages_round_trip() {
        for message in [
            ClientMessage::Cast,
            ClientMessage::Select(-2),
            ClientMessage::Sync,
        ] {
            assert_eq!(ClientMessage::decode(&message.encode()), Some(message));
        }
    }

    #[test]
    fn junk_from_a_client_is_ignored() {
        assert_eq!(ClientMessage::decode(""), None);
        assert_eq!(ClientMessage::decode("cast everything"), None);
        assert_eq!(ClientMessage::decode("select many"), None);
        assert_eq!(ClientMessage::decode("select 100000"), None);
        assert_eq!(ClientMessage::decode("give me mana"), None);
    }

    #[test]
    fn a_snapshot_round_trips() {
        let snapshot = StateSnapshot {
            level: 12,
            xp_into: 40,
            xp_needed: 1400,
            mana: 88.5,
            mana_max: 148.0,
            resonance: 35.5,
            cooldown: 14,
            spell: "Fireball".to_owned(),
            element: "pyro".to_owned(),
        };
        let decoded = StateSnapshot::decode(&snapshot.encode()).expect("round trip");
        assert_eq!(decoded, snapshot);
    }

    #[test]
    fn a_snapshot_from_a_future_version_is_refused() {
        assert_eq!(StateSnapshot::decode("v2;level=1"), None);
        assert_eq!(StateSnapshot::decode("garbage"), None);
    }

    #[test]
    fn an_unknown_key_is_ignored() {
        let line = "v1;level=2;xp=1/100;mana=10.0/20.0;res=0.0;cd=0;spell=;el=;extra=yes";
        let decoded = StateSnapshot::decode(line).expect("forward compatible");
        assert_eq!(decoded.level, 2);
        assert!(decoded.spell.is_empty());
    }

    #[test]
    fn ratios_are_bounded() {
        let snapshot = StateSnapshot {
            level: 1,
            xp_into: 500,
            xp_needed: 100,
            mana: 500.0,
            mana_max: 100.0,
            resonance: 500.0,
            cooldown: 0,
            spell: String::new(),
            element: String::new(),
        };
        assert_eq!(snapshot.mana_ratio(), 1.0);
        assert_eq!(snapshot.xp_ratio(), 1.0);
        assert_eq!(snapshot.resonance_ratio(), 1.0);
    }

    #[test]
    fn a_missing_setting_means_medium() {
        assert_eq!(quality_from_setting(None), RenderQuality::Medium);
        assert_eq!(quality_from_setting(Some("high")), RenderQuality::High);
        assert_eq!(quality_from_setting(Some("nope")), RenderQuality::Medium);
    }
}
