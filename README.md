# ARCANUM

**The power of magic is in your hands.**

Arcanum is an RPG magic and spellcraft mod for [Horizon](https://github.com/horizon-rust/horizon),
the Minecraft: Java Edition reimplementation written in Rust. It is a **native
Horizon mod** — a `horizon-mixins` bundle written entirely in Rust — not a
Java mod, and nothing here runs on Forge, NeoForge, Fabric or the vanilla
launcher.

> Horizon is not an official Minecraft product and neither is Arcanum. Arcanum
> contains no Minecraft code, assets or data; it calls Horizon's own APIs and
> vanilla content.

- **Platform:** Horizon only (native mods, `.hnmod`)
- **Language:** Rust, one dependency-free engine crate plus a dozen hook helpers
- **Status:** 0.1.0 — vertical slice: mana, a spellbook of six schools, an
  authoritative cast pipeline, a HUD, commands, settings and saved progression

## What works

| Feature | State |
| --- | --- |
| Mana pool, regeneration, resonance | Implemented, unit-tested in the engine |
| Six schools with their own particles, sounds, damage types and colours | Implemented, unit-tested |
| Modular spell builder with hard caps and an efficiency floor | Implemented, unit-tested against every 3-modifier combination |
| 30 built-in spells (5 per school) | Implemented, balanced in `mixins/arcanum/engine/src/registry.rs` |
| Server-authoritative casting | Implemented: the client sends *cast*; the server checks the spell, the mana, the cooldown and the level |
| HUD: mana, experience, resonance, selected spell, cooldown | Implemented against Horizon's `native_hud` hook |
| Keys and rebinding (Options → Mods) | Implemented against `native_key` + `native_mod_key` |
| Grimoire, `/arcanum` commands, action-bar feedback | Implemented against `native_register_commands` / `native_perform_command` |
| Progression: 50 levels, 5 stats, mastery, talents, a specialization | Implemented, unit-tested |
| Saved progression per player | Text file per player under `config/arcanum/players/` |
| Settings page (HUD on/off, particle quality) | Implemented against `native_settings` |

## What is not there yet

Read [`docs/API_LIMITATIONS.md`](docs/API_LIMITATIONS.md) before expecting any
of this: a radial spell wheel *screen* (Horizon has no API for a mod's own
screen outside mod block menus, so the wheel is three keys), staffs, robes and
trinkets (content mods are a separate `.hmod` download), bosses and structures,
and a spell-crafting *screen* (the engine accepts any composed spell, the UI to
compose one is not built).

## Verified, and not verified

Be precise about this, because it is the difference between a mod and a sketch.

**Verified in this repository, by `cargo test` (53 unit tests + 60 integration
assertions across 6 test files):** every number in the engine — the mana
economy, the spell builder's refusals, all 30 spells against the caps, the
progression curve, the cast-to-command translation, the save format and the mod
message codecs.

**Not verified here:** the hook glue (`runtime.rs` and the helper files beside
it) compiles only inside a Horizon checkout, and building Horizon needs its
full dependency tree — this repository was developed in a 2-core sandbox that
cannot build it. The glue was written against the exact signatures of one
audited commit (`compatibility.json`), and every type and method it calls was
read from that commit, but *it has not been compiled against the game here*.
Run the staging check below before you trust it, and read the report in
[`docs/API_LIMITATIONS.md`](docs/API_LIMITATIONS.md).

## Install (players)

Native mods are built into a game of their own by the Horizon launcher:

1. Install a Horizon launcher with native-mod support and an **21.0.0**-or-newer
   Nightly that reports hook API 42.
2. Launcher → **Mods** → install `arcanum-0.1.0.hnmod`, or unpack it into
   `<instance>/horizon-native-mods/`.
3. Settings → **Native mods (experimental)**, then **Build** on the instance.
   The first build downloads Rust and takes a while.
4. Play the *Native mods* instance. In game, **Options → Mods** lists Arcanum,
   its two settings and its three keys (`R` cast, `V` next spell, `B` previous).

Native mods run with your own rights and are not sandboxed. Build only mods
whose source you trust — this one is MIT and short enough to read.

## Build and test (developers)

The engine is a plain crate with no dependencies:

```sh
cargo test --workspace          # the whole verified surface
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo run -p arcanum-engine --example spellbook    # prints docs/SPELLS.md's tables
python3 tools/package.py                           # target/arcanum-0.1.0.hnmod
```

The bundle has to be compiled *into* Horizon, which is what the mixin tool
does:

```sh
git clone https://github.com/horizon-rust/horizon ../horizon
git -C ../horizon checkout 83e9b542b89f78a523115252c9a81e9916611788
cargo install --git https://github.com/horizon-rust/horizon-mixins \
  --rev d9c83d7096b7ea5b83678da2efdfd6161f74997e horizon-mixins-cli

horizon-mixins plan  --workspace ../horizon --mods mixins --out ../arcanum-plan.json
horizon-mixins check --workspace ../horizon --mods mixins --out ../arcanum-build --trust-native-mods
```

`plan` resolves each named hook point against Horizon's `native-hooks.json` and
refuses a name or a version it does not have. `check` stages the modded
workspace in `../arcanum-build` and compiles it; it never edits `../horizon`.
Then run the game the way Horizon's own docs describe, with prepared resources,
and cast a spell.

## Layout

```
arcanum-horizon/
├── Cargo.toml                     workspace: the engine crate, for tests
├── compatibility.json             the audited Horizon commit and file hashes
├── mixins/arcanum/                the mod (this is what ships)
│   ├── horizon-mixin.toml         bundle manifest: module, crate, 12 hook points
│   ├── runtime.rs                 injected module: state, casting, persistence
│   ├── hud.rs ... player_died.rs  one thin helper per hook point
│   └── engine/                    arcanum-engine: all the game logic
│       ├── src/                   mana, elements, shapes, modifiers, spell,
│       │                          spell_builder, registry, cast, player,
│       │                          progression, codec, net
│       └── tests/                 mana, spell, balance, progression, cast
├── tools/package.py               builds the .hnmod
└── docs/                          spells, balance, multiplayer, limitations
```

The split is Horizon's own recommendation: heavy logic in the bundle's crate
(compiled once, kept across rebuilds), glue in helpers compiled into the game.

## Design

A cast is a pure function. `PlayerState::cast` validates the spell, the level,
the mana, the cooldown and the caster's name, and only then spends anything;
`cast::resolve` turns the spell into the vanilla commands the server runs as
its console. Nothing trusts the client: it can send "cast" and "select", and
nothing else. See [`docs/MULTIPLAYER.md`](docs/MULTIPLAYER.md).

Everything derived — cost, power, range, cooldown — comes from
[`mixins/arcanum/engine/src/balance.rs`](mixins/arcanum/engine/src/balance.rs)
and [`docs/BALANCE.md`](docs/BALANCE.md). Adding a spell is one line in
`registry.rs`; the builder validates it and `tests/balance_tests.rs` fails if it
breaks a cap.

## Licence

MIT. Horizon is MIT too. Not an official Minecraft product.
