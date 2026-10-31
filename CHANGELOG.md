# Changelog

All notable changes to Arcanum. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — 2026-10-31

The first vertical slice: a mage can learn spells, cast them, and grow.

### Added

- `arcanum-engine`, a dependency-free crate holding the whole game logic and
  every test that can run without the game:
  - the six schools (pyromancy, cryomancy, electromancy, nature, shadow,
    arcane), each with its own HUD colour, vanilla particle, vanilla sound and
    vanilla damage type;
  - seven shapes, seven effects and nine modifiers, with conflicts;
  - the modular spell builder, with hard caps on cost, power, range and
    cooldown, and an efficiency floor that refuses any spell cheaper than 90%
    of its own power;
  - 30 built-in spells, five per school, with level requirements;
  - mana with regeneration, resonance with decay, and per-spell cooldowns;
  - 50 levels, five stats, elemental mastery, twelve talents and a
    specialization;
  - `PlayerState`, the only place a cast may start: it checks the spell, the
    level, the mana, the cooldown and the caster's name, and spends nothing
    when any of them fails;
  - `cast::resolve`, which turns a cast into the vanilla commands the
    authoritative server runs, so every school keeps its visual identity;
  - a versioned, diffable save format and the client/server message codecs.
- A Horizon native-mod bundle (`mixins/arcanum`): one support module, the
  engine crate, and twelve HEAD hooks on named hook points — `hud`, `key`,
  `tick`, `player_tick`, `net_from_client`, `net_from_server`,
  `register_commands`, `perform_command`, `mods`, `settings`, `settings_click`,
  `player_died`.
- HUD (mana, experience, resonance, selected spell, cooldown), three rebindable
  keys, `/arcanum` commands, a settings page, per-player saves under
  `config/arcanum/players/`.
- `tools/package.py`, which writes a reproducible `arcanum-0.1.0.hnmod`.
- Documentation: `docs/SPELLS.md`, `docs/BALANCE.md`, `docs/MULTIPLAYER.md`,
  `docs/API_LIMITATIONS.md`.

### Notes

- The engine is verified by `cargo test`. The hook glue has **not** been
  compiled against Horizon in this environment; see
  `docs/API_LIMITATIONS.md` for exactly what that means and how to check it.
- No spell-creation screen, no equipment and no structures yet: Horizon has no
  API for a mod's own screen outside mod block menus, and equipment would be a
  content mod.
