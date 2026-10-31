# Horizon API limitations, and what Arcanum does about them

This is the honest list. Everything here was checked against Horizon's own
source at the commit in `compatibility.json` (`83e9b54`, hook API 42), not
against a guess about what a Minecraft mod usually can do.

## How a native mod reaches the game

Horizon has no plugin interface, no event bus and no `unsafe` entry point. A
native mod is a **`horizon-mixins` bundle**: a `horizon-mixin.toml` plus Rust
source that the tool splices into Horizon's own files before they are compiled.
Arcanum uses only *named hook points* (`native-hooks.json`), which Horizon keeps
stable across releases and versions by contract. That is the whole of the API
surface below.

## What Arcanum uses, and what it buys

| Hook / helper | API | What Arcanum does with it |
| --- | --- | --- |
| `native_hud(batch, corners)` | 1 | Draws the mana, experience and resonance bars and the selected spell |
| `native_key(key, pressed)` | 1 | Reads the three keys (`native_mod_key` for the player's binding) |
| `native_tick()` | 1 | Counts the cooldown down between server syncs |
| `native_player_tick()` | 31 | Per-player, per-tick: mana, resonance, cooldowns, levels, state sync |
| `native_net_from_client(player, channel, data)` | 40 | The only way a cast starts: one "cast" message |
| `native_net_from_server(channel, data)` | 40 | The state the HUD draws |
| `native_register_commands` / `native_perform_command` | 8 | `/arcanum` on the server, with op gating |
| `native_mods` | 1 | Options → Mods entry with three rebindable keys |
| `native_settings` / `native_settings_click` | 1 | HUD on/off, particle quality |
| `native_player_died` | 40 | Empties mana and resonance, keeps progression |
| `native_run_command(line)` | 21 | Runs the vanilla commands a cast resolves into |
| `native_action_bar`, `native_feedback`, `native_chat` | 21 / 8 / 1 | Feedback to the player |
| `native_config_dir()` | 26 | `config/arcanum/players/<uuid>.arc` |
| `native_on`, `native_setting`, `native_set_setting` | 1 | The mod's switch and settings |

## Not available, and the workaround chosen

**No raycast, no entity search, no direct damage API.** Horizon exposes
`native_mobs` (API 37) for spawning and reading mobs, but there is no "what is
this player looking at". Arcanum therefore resolves a cast into vanilla
commands — `execute at <caster> run damage @e[distance=..R,sort=nearest,...]` —
which the authoritative server runs as its console. This is a real Horizon API
(`native_run_command`), not a simulation: the commands are vanilla syntax and
Horizon implements every vanilla command. It also means a cast affects the
world through the same rules a player's own command would.

*Consequence:* the target is "the nearest mob within range", not "what the
crosshair is on". A precise aim needs a raycast API Horizon does not have yet.

**No custom damage types.** Damage is dealt as a vanilla type per school
(`on_fire`, `freeze`, `lightning_bolt`, `magic`, `wither`, `indirect_magic`).
The death message therefore says "burned to death", not "was consumed by
pyromancy".

**No mod screen.** Horizon lets a mod draw its own screen *for a mod block's
menu* (`native_screen`, API 19) and a panel beside container screens
(`native_panel_items`, API 2). There is no way to open a screen of your own from
a key. So:

- the "spell wheel" is three keys (cast, next, previous) and a HUD, not a radial
  menu;
- the skill tree and the spell-crafting UI exist in the engine (level gates,
  talents, `SpellBuilder` with its refusals) but have no screen. The engine is
  what a future screen would call; today they are reachable through
  `/arcanum`.

**No items of our own.** Items need a *content mod* (`.hmod`): data and assets
in a separate file, no code, and content mods work in single player today.
Staffs, grimoires, rings, amulets, robes and mana crystals are therefore a
second download that Arcanum would read through `native_data` /
`native_recipes_of_type` (API 36) once it exists. Nothing in this repository
pretends they are implemented.

**No rendering hook of our own.** Only the HUD (`native_hud`) and mod-block
screens. Particles and sounds are emitted by the authoritative server through
vanilla commands, which is why every school's identity travels with the cast
rather than with the client's rendering settings. `GameLevel::native_send_particles`
(API 11) and `native_play_sound` (API 25) exist and would be the cheaper path
for a future version — they need a `GameLevel` binding that the per-player hooks
do not hand out, which is why this version uses commands.

**No health or movement writes.** A mod cannot set a player's health, so
Vitality does not raise it: it raises the mana pool (see `BALANCE.md`).
Shields, roots and slows go through vanilla effects, which the game applies.

**Multiplayer is a Horizon server, not a mod server.** Mod messages work
between a mod's server half and its client half on a LAN host or a dedicated
server (API 40). A vanilla server has no mod messages, and Arcanum says so in
chat instead of pretending a cast happened. See `MULTIPLAYER.md`.

## What has not been verified, and how to verify it

The engine is covered by `cargo test`. The glue — `runtime.rs` and the twelve
helper files — compiles only inside Horizon, and building Horizon needs its
full dependency tree (a Rust toolchain, then a multi-gigabyte build). The
sandbox this mod was written in is a 2-core machine that cannot build it, so:

- **not verified:** that the glue compiles against `83e9b54`, and that a cast
  looks and feels right in game;
- **verified by reading, not compiling:** every hook name, version, signature
  and helper used above, read from that commit's source;
- **verified by test:** every number, every refusal and every command string.

To close the gap, run the staging check (see `README.md`):

```sh
horizon-mixins plan  --workspace ../horizon --mods mixins --out ../arcanum-plan.json
horizon-mixins check --workspace ../horizon --mods mixins --out ../arcanum-build --trust-native-mods
```

`plan` fails with `E_HOOK_UNKNOWN` or `E_HOOK_VERSION` if a hook name or version
is wrong. `check` runs `cargo test` on the staged workspace — a real type check
of the glue against the game. If it fails, the error names the line; the fix
belongs in this repository, and the README asks you to report it.

## Version pinning

`horizon-mixin.toml` pins `horizon_revision`; `horizon-native.json` (written by
`tools/package.py`) also names `hooks: 42`, so the launcher may build Arcanum on
a later Nightly whose hook API is still at least 42. Hook contracts only change
by adding a version, so that is safe by Horizon's own rule.
