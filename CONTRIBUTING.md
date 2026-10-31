# Contributing to Arcanum

Arcanum is a native Horizon mod. That shapes every rule below: there is no
Java, no Forge, no Fabric, no Mixin and no Minecraft class to patch. If a
change needs an API Horizon does not have, the answer is a documented
limitation, not an invented function.

## Ground rules

1. **Horizon only.** Rust, `horizon-mixins` bundles, hook points from
   `native-hooks.json`. Never add Java, a `.jar`, a Forge/Fabric dependency or
   a Mixin.
2. **No invented API.** Every game function the glue calls must exist at the
   commit in `compatibility.json`. If you need something new, open an issue on
   Horizon first and pin the commit that adds it.
3. **The engine stays dependency-free.** Horizon's `Cargo.lock` is the allow
   list for a bundle crate's dependencies, and the engine is testable only
   because it needs nothing. If a dependency is genuinely required, say which
   locked version it is and why in the pull request.
4. **The server owns the numbers.** Anything a client sends is intent, never
   authority: mana, cooldowns, known spells, damage and effects are decided in
   `PlayerState` on the server.
5. **Keep it compiled and tested.** `cargo test --workspace`,
   `cargo clippy --workspace --all-targets -- -D warnings` and
   `cargo fmt --all --check` must pass. Add a test with the behaviour.

## Adding a spell

1. Add one line to `DEFS` in `mixins/arcanum/engine/src/registry.rs`.
2. Run `cargo test --workspace`. The balance tests refuse anything over the
   caps, below the efficiency floor, or with a shape/effect pair that cannot
   work.
3. Regenerate the tables in `docs/SPELLS.md`:
   `cargo run -p arcanum-engine --example spellbook`.

## Adding an element, a shape, an effect or a modifier

The engine is built so new ones do not touch the cast pipeline:

- **Modifier:** add a variant to `modifiers.rs` with its `cost_delta`,
  `power_bonus`, `cost_scale`, `range_scale`, `cooldown_delta` and any
  conflicts. Nothing else changes: `Spell::derive` reads those five numbers.
- **Shape:** add a variant to `shapes.rs` with its base numbers and
  `targeting()`, then one arm in `cast::add_visuals` if its visuals differ.
- **Effect:** add a variant to `effects.rs` with its base numbers, then one arm
  in `cast::resolve` that produces the vanilla commands it means.
- **Element:** add a variant to `elements.rs`; the compiler lists every place
  that needs a colour, a particle, a sound, a damage type and two multipliers.

If a new element would need a change to `Spell::derive`, the change is in the
wrong place — tell us in the pull request.

## Changing the bundle's glue

`runtime.rs`, `hud.rs`, `key.rs` and the rest are compiled inside
`crates/horizon-client`. Two things to respect:

- A module payload may not use inner doc comments (`//!`), `mod` declarations
  or `include!`. Use `//` comments.
- A helper's parameter types must match the hook's own signature exactly; the
  binding in `horizon-mixin.toml` is what the tool substitutes.

Verify with `horizon-mixins check` (see `README.md`); a plan that resolves is
not a compile.

## Pull requests

- One topic per pull request, small and reviewable.
- Say what you verified and how: `cargo test`, a staging check, or "not
  verified, needs a Horizon build".
- Never claim a feature works in game when only the engine tests passed.
- Update `CHANGELOG.md` under *Unreleased*.
