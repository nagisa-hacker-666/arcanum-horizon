// Arcanum's runtime: state, casting, persistence, commands and settings.
//
// horizon-mixins injects this file into `crates/horizon-client/src/game.rs` as
// the module `crate::game::arcanum`, and the hook helpers beside it
// (`hud.rs`, `key.rs`, ...) call into it. Module payloads cannot use inner doc
// comments, `mod` declarations or `include!`, so everything here is one flat
// module of ordinary items.
//
// The server half owns every number a cast depends on. The client half only
// draws what the server sent and forwards the player's intent, so a modified
// client cannot invent mana, spells or cooldowns.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use arcanum_engine::net::{quality_from_setting, ClientMessage, StateSnapshot};
use arcanum_engine::{
    codec, PlayerState, RenderQuality, SpellRegistry, CHANNEL_CAST, CHANNEL_STATE, MOD_ID,
};

/// The key names Options > Mods lists, with their default bindings.
const KEYS: [(&str, &str, &str); 3] = [
    ("cast", "Cast spell", "key.keyboard.r"),
    ("next", "Next spell", "key.keyboard.v"),
    ("prev", "Previous spell", "key.keyboard.b"),
];

static REGISTRY: OnceLock<SpellRegistry> = OnceLock::new();
static PLAYERS: Mutex<BTreeMap<u128, PlayerState>> = Mutex::new(BTreeMap::new());
static CLIENT: Mutex<Option<StateSnapshot>> = Mutex::new(None);
static TICKS: AtomicU64 = AtomicU64::new(0);
static NET_HINTED: Mutex<bool> = Mutex::new(false);

fn registry() -> &'static SpellRegistry {
    REGISTRY.get_or_init(|| {
        SpellRegistry::builtin().unwrap_or_else(|error| {
            eprintln!("[arcanum] the spellbook did not build: {error}");
            SpellRegistry::empty()
        })
    })
}

fn players() -> MutexGuard<'static, BTreeMap<u128, PlayerState>> {
    PLAYERS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

// Persistence: one text file per player under config/arcanum/players/. Saving
// is a plain file write: native mods run with the player's own rights, and
// Game::native_config_dir (hook API 26) is the folder mods share.
fn players_dir() -> Option<PathBuf> {
    let dir = crate::game::Game::native_config_dir()
        .ok()?
        .join("arcanum")
        .join("players");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn save_path(uuid: u128) -> Option<PathBuf> {
    Some(players_dir()?.join(format!("{uuid:032x}.arc")))
}

fn save(state: &PlayerState) {
    let Some(path) = save_path(state.uuid) else {
        return;
    };
    if let Err(error) = std::fs::write(&path, codec::encode(state)) {
        eprintln!("[arcanum] could not save {}: {error}", path.display());
    }
}

fn load(uuid: u128, name: &str) -> Option<PlayerState> {
    let path = save_path(uuid)?;
    let text = std::fs::read_to_string(&path).ok()?;
    match codec::decode(&text) {
        Ok(mut state) => {
            state.name = name.to_owned();
            Some(state)
        }
        Err(error) => {
            eprintln!("[arcanum] could not read {}: {error}", path.display());
            None
        }
    }
}

/// Every spell the mage's level unlocks: spells are earned, never granted.
fn sync_grimoire(state: &mut PlayerState) {
    let level = state.progression.level;
    for spell in registry().learnable(level) {
        let _ = state.learn(spell);
    }
}

/// Takes the mage out of the shared map, loading them from disk the first
/// time. The caller puts them back with [`store`], so no lock is held while a
/// hook talks to the game.
fn state_of(uuid: u128, name: &str) -> PlayerState {
    if let Some(state) = players().remove(&uuid) {
        return state;
    }
    let mut state = load(uuid, name).unwrap_or_else(|| PlayerState::new(uuid, name));
    sync_grimoire(&mut state);
    state
}

fn store(state: PlayerState) {
    players().insert(state.uuid, state);
}

fn snapshot_of(state: &PlayerState) -> StateSnapshot {
    let spell = state.selected_spell(registry());
    let (xp_into, xp_needed) = state.progression.level_progress();
    StateSnapshot {
        level: state.progression.level,
        xp_into,
        xp_needed,
        mana: state.mana.current(),
        mana_max: state.mana.max(),
        resonance: state.resonance.value(),
        cooldown: spell.map_or(0, |spell| state.cooldowns.remaining(&spell.id)),
        spell: spell.map_or_else(String::new, |spell| spell.name.clone()),
        element: spell.map_or_else(String::new, |spell| spell.element.id().to_owned()),
    }
}

fn send_snapshot(game: &mut crate::game::Game<'_>, player: u128, state: &PlayerState) {
    let snapshot = snapshot_of(state);
    if let Err(error) =
        game.native_net_send_to_player(player, CHANNEL_STATE, snapshot.encode().as_bytes())
    {
        eprintln!("[arcanum] could not sync {}: {error}", state.name);
    }
}

fn tell(game: &mut crate::game::Game<'_>, player: u128, line: &str) {
    game.native_action_bar(player, crate::text::Text::literal(line));
}

fn setting(game: &crate::game::Game<'_>, key: &str) -> Option<String> {
    game.native_setting(MOD_ID, key)
}

fn hud_enabled(game: &crate::game::Game<'_>) -> bool {
    setting(game, "hud").as_deref() != Some("off")
}

fn quality(game: &crate::game::Game<'_>) -> RenderQuality {
    quality_from_setting(setting(game, "quality").as_deref())
}

fn binding(game: &mut crate::game::Game<'_>, key: &str) -> String {
    let index = KEYS.iter().position(|entry| entry.0 == key).unwrap_or(0);
    let entry = KEYS[index];
    game.native_mod_key(MOD_ID, entry.0, entry.2)
}

/// A key or mouse button went down or up: cast, or move the selection.
pub(crate) fn on_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    if !pressed || !game.native_on(MOD_ID) || !game.native_playing() {
        return;
    }
    if key == binding(game, "cast") {
        request(game, ClientMessage::Cast);
    } else if key == binding(game, "next") {
        request(game, ClientMessage::Select(1));
    } else if key == binding(game, "prev") {
        request(game, ClientMessage::Select(-1));
    }
}

/// Asks the server to do something. The client never decides it happened.
fn request(game: &mut crate::game::Game<'_>, message: ClientMessage) {
    let text = message.encode();
    if let Err(error) = game.native_net_send_to_server(CHANNEL_CAST, text.as_bytes()) {
        let mut hinted = NET_HINTED
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !*hinted {
            *hinted = true;
            game.native_chat(&format!(
                "[Arcanum] {error}. Casting needs a Horizon server that carries mod messages."
            ));
        }
    }
}

/// Once a tick, on the window: count the cooldown down between snapshots, so
/// the HUD does not look frozen.
pub(crate) fn on_tick(_game: &mut crate::game::Game<'_>) {
    let mut slot = CLIENT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(snapshot) = slot.as_mut() {
        snapshot.cooldown = snapshot.cooldown.saturating_sub(1);
    }
}

/// The authoritative per-player tick: mana, resonance, cooldowns, levels, and
/// the state sync the HUD draws.
pub(crate) fn on_player_tick(game: &mut crate::game::Game<'_>) {
    if !game.native_on(MOD_ID) {
        return;
    }
    let Some(player) = game.native_acting_player() else {
        return;
    };
    let mut state = state_of(player.uuid, &player.name);
    let before = state.progression.level;
    state.tick(1);
    if state.progression.level > before {
        sync_grimoire(&mut state);
        tell(
            game,
            player.uuid,
            &format!("Arcanum: level {}", state.progression.level),
        );
    }
    if TICKS.fetch_add(1, Ordering::Relaxed) % 20 == 0 {
        send_snapshot(game, player.uuid, &state);
        save(&state);
    }
    store(state);
}

/// A message a player's client sent: the only way a cast starts.
pub(crate) fn on_net_from_client(
    game: &mut crate::game::Game<'_>,
    player: u128,
    channel: &str,
    data: &[u8],
) {
    if channel != CHANNEL_CAST || !game.native_on(MOD_ID) {
        return;
    }
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Some(message) = ClientMessage::decode(text) else {
        return;
    };
    let Some(info) = game
        .native_server_players()
        .into_iter()
        .find(|info| info.uuid == player)
    else {
        return;
    };
    let mut state = state_of(player, &info.name);
    let mut line = String::new();
    match message {
        ClientMessage::Cast => match state.try_cast(registry(), quality(game)) {
            Ok(plan) => {
                for command in &plan.commands {
                    game.native_run_command(command);
                }
                line = plan.feedback;
            }
            Err(error) => line = error.to_string(),
        },
        ClientMessage::Select(delta) => match state.cycle(delta) {
            Some(id) => {
                line = format!(
                    "Selected: {}",
                    registry().get(id).map_or(id, |spell| spell.name.as_str())
                );
            }
            None => line = "Your grimoire is empty.".to_owned(),
        },
        ClientMessage::Sync => {}
    }
    if !line.is_empty() {
        tell(game, player, &line);
    }
    send_snapshot(game, player, &state);
    save(&state);
    store(state);
}

/// A message the server sent the window.
pub(crate) fn on_net_from_server(_game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) {
    if channel != CHANNEL_STATE {
        return;
    }
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Some(snapshot) = StateSnapshot::decode(text) else {
        return;
    };
    *CLIENT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(snapshot);
}

/// The commands Arcanum adds to the server's tree.
pub(crate) fn on_register_commands(
    _game: &mut crate::game::Game<'_>,
    commands: &mut Vec<crate::game::native_server::NativeCommand>,
) {
    use crate::game::native_server::NativeCommand;
    for usage in [
        "arcanum",
        "arcanum status",
        "arcanum grimoire",
        "arcanum learn <spell:word>",
        "arcanum select <slot:integer>",
    ] {
        commands.push(NativeCommand::new(usage));
    }
    commands.push(NativeCommand::op("arcanum grant <spell:word>", 2));
    commands.push(NativeCommand::op("arcanum reset", 2));
}

/// A player's Arcanum command, run on the server.
pub(crate) fn on_perform_command(
    game: &mut crate::game::Game<'_>,
    player: &crate::game::native_server::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    let mut words = command.split_whitespace();
    if words.next() != Some(MOD_ID) {
        return;
    }
    *ran = true;
    let mut state = state_of(player.uuid, &player.name);
    let mut line = format!(
        "Arcanum {}: level {}, {:.0}/{:.0} mana, {} spells",
        arcanum_engine::VERSION,
        state.progression.level,
        state.mana.current(),
        state.mana.max(),
        state.known().len()
    );
    match words.next() {
        None | Some("status") => {}
        Some("grimoire") => line = grimoire_line(&state),
        Some("learn") => {
            line = match words.next().and_then(|id| registry().get(id)) {
                Some(spell) => {
                    let spell = spell.clone();
                    match state.learn(&spell) {
                        Ok(()) => format!("Learned {}.", spell.name),
                        Err(error) => error.to_string(),
                    }
                }
                None => "No such spell: try `arcanum grimoire`.".to_owned(),
            };
        }
        Some("select") => {
            line = match words.next().and_then(|word| word.parse::<usize>().ok()) {
                Some(index) => match state.select(index) {
                    Some(id) => format!(
                        "Selected: {}",
                        registry().get(id).map_or(id, |spell| spell.name.as_str())
                    ),
                    None => format!("There is no spell {}.", index + 1),
                },
                None => "Usage: arcanum select <slot>".to_owned(),
            };
        }
        Some("grant") => {
            if player.permission >= 2 {
                line = match words.next().and_then(|id| registry().get(id)) {
                    Some(spell) => {
                        let spell = spell.clone();
                        match state.learn(&spell) {
                            Ok(()) => format!("Granted {}.", spell.name),
                            Err(error) => error.to_string(),
                        }
                    }
                    None => "No such spell.".to_owned(),
                };
            } else {
                line = "You are not allowed to use this command.".to_owned();
            }
        }
        Some("reset") => {
            if player.permission >= 2 {
                state = PlayerState::new(player.uuid, &player.name);
                sync_grimoire(&mut state);
                line = "Your Arcanum progress has been reset.".to_owned();
            } else {
                line = "You are not allowed to use this command.".to_owned();
            }
        }
        Some(other) => line = format!("Unknown: arcanum {other}"),
    }
    game.native_feedback(crate::text::Text::literal(&line));
    save(&state);
    send_snapshot(game, player.uuid, &state);
    store(state);
}

fn grimoire_line(state: &PlayerState) -> String {
    let mut text = String::from("Grimoire:");
    for (index, id) in state.known().iter().enumerate() {
        if let Some(spell) = registry().get(id) {
            text.push_str(&format!(" {}) {}", index + 1, spell.name));
        }
    }
    if state.known().is_empty() {
        text.push_str(" (empty)");
    }
    text
}

/// Options > Mods lists Arcanum with its keys.
pub(crate) fn on_mods(
    _game: &mut crate::game::Game<'_>,
    mods: &mut Vec<crate::game::native_hooks::NativeMod>,
) {
    use crate::game::native_hooks::NativeMod;
    let mut entry = NativeMod::default();
    entry.id = MOD_ID.to_owned();
    entry.title = format!("Arcanum {}", arcanum_engine::VERSION);
    entry.description =
        "Six schools of magic, a modular spell builder and a mage's progression.".to_owned();
    entry.keys = KEYS
        .iter()
        .map(|(name, label, default)| {
            [
                (*name).to_owned(),
                (*label).to_owned(),
                (*default).to_owned(),
            ]
        })
        .collect();
    mods.push(entry);
}

/// The buttons on Arcanum's settings page.
pub(crate) fn on_settings(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    buttons: &mut Vec<String>,
) {
    if mod_id != MOD_ID {
        return;
    }
    buttons.push(format!(
        "HUD: {}",
        if hud_enabled(game) { "on" } else { "off" }
    ));
    buttons.push(format!("Particles: {}", quality(game)));
}

pub(crate) fn on_settings_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    if mod_id != MOD_ID {
        return;
    }
    match button {
        0 => {
            let next = if hud_enabled(game) { "off" } else { "on" };
            game.native_set_setting(MOD_ID, "hud", next);
        }
        1 => {
            let next = quality(game).next();
            game.native_set_setting(MOD_ID, "quality", next.id());
        }
        _ => {}
    }
}

/// Dying empties the mana and the resonance; the progression stays.
pub(crate) fn on_player_died(game: &mut crate::game::Game<'_>, player: u128, _cause: Option<&str>) {
    let Some(info) = game
        .native_server_players()
        .into_iter()
        .find(|info| info.uuid == player)
    else {
        return;
    };
    let mut state = state_of(player, &info.name);
    state.mana.empty();
    state.resonance.clear();
    state.cooldowns.clear();
    state.refresh();
    send_snapshot(game, player, &state);
    save(&state);
    store(state);
}

/// The snapshot the HUD draws, or none before the first sync.
pub(crate) fn snapshot() -> Option<StateSnapshot> {
    CLIENT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

pub(crate) fn hud_visible(game: &crate::game::Game<'_>) -> bool {
    game.native_on(MOD_ID) && hud_enabled(game)
}

/// The magic HUD: mana, experience, resonance, the selected spell and its
/// cooldown, drawn in a box `Corners` places so it never covers another mod's.
pub(crate) fn on_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut crate::hud::HudBatch,
    corners: &mut crate::extras_hud::Corners,
) {
    if !game.native_playing() || !hud_visible(game) {
        return;
    }
    let Some(state) = snapshot() else {
        return;
    };
    let corner = crate::extras::Corner::BottomLeft;
    if !game.native_corner_shown(corner) {
        return;
    }
    let accent = match arcanum_engine::Element::from_id(&state.element) {
        Some(element) => 0xFF00_0000 | element.color(),
        None => 0xFF9A_A7FF,
    };
    let (width, height) = (126.0, 32.0);
    let (x, y) = corners.place(corner, width, height);
    game.native_fill(batch, [x, y, width, height], 0x7800_0000);

    let spell = if state.spell.is_empty() {
        "no spell"
    } else {
        state.spell.as_str()
    };
    let line = if state.is_ready() {
        format!("Lv {} {}", state.level, spell)
    } else {
        format!(
            "Lv {} {} {:.1}s",
            state.level,
            spell,
            state.cooldown as f32 / 20.0
        )
    };
    game.native_text(batch, &line, [x + 3.0, y + 3.0], accent, true);

    // Mana, experience, then resonance once a mage has any.
    game.native_fill(batch, [x + 3.0, y + 14.0, width - 6.0, 6.0], 0x5000_0000);
    game.native_fill(
        batch,
        [x + 3.0, y + 14.0, (width - 6.0) * state.mana_ratio(), 6.0],
        accent,
    );
    game.native_fill(batch, [x + 3.0, y + 22.0, width - 6.0, 3.0], 0x5000_0000);
    game.native_fill(
        batch,
        [x + 3.0, y + 22.0, (width - 6.0) * state.xp_ratio(), 3.0],
        0xFF7C_E87C,
    );
    if state.resonance > 0.0 {
        game.native_fill(
            batch,
            [
                x + 3.0,
                y + 27.0,
                (width - 6.0) * state.resonance_ratio(),
                2.0,
            ],
            0xFFB0_7CFF,
        );
    }
    let mana = format!("{:.0}/{:.0}", state.mana, state.mana_max);
    let right = x + width - 3.0 - game.native_text_width(&mana);
    game.native_text(batch, &mana, [right, y + 3.0], 0xFFFF_FFFF, true);
}
