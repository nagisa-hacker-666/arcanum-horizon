# The Arcanum spellbook

Thirty spells, five per school. Every row below is generated from
`mixins/arcanum/engine/src/registry.rs` by

```sh
cargo run -p arcanum-engine --example spellbook
```

so this file cannot drift from the code. Cooldowns are shown in seconds (the
engine counts ticks, 20 to the second); "Mana" is the base cost before the
caster's Mastery; "Power" is the base power before the caster's Power and
mastery.

## How a spell is put together

A spell is an **element** (the six schools), a **shape** (how it reaches its
target), an **effect** (what happens when it lands) and up to **three
modifiers**. The builder derives the rest — cost, power, range, cooldown — and
refuses anything past a cap or below the efficiency floor
(`docs/BALANCE.md`).

| Shapes | Targets | | Effects | Does |
| --- | --- | --- | --- | --- |
| Projectile | nearest enemy | | Damage | deals damage as the school's type |
| Beam | nearest enemy, further | | Heal | restores health (self or area) |
| Zone | every enemy around | | Slow | slowness I for a few seconds |
| Nova | every enemy around, harder | | Root | slowness VI, held in place |
| Wall | wide and short | | Shield | absorption + resistance on the caster |
| Self | the caster | | Drain | damage + instant health to the caster |
| Blink | the caster, displaced | | Teleport | a step through space |

| Modifiers | Effect |
| --- | --- |
| Explosive | +45% power, +14 mana |
| Chaining | +25% power, +16 mana, +10 ticks |
| Piercing | +20% power, +20% range, +10 mana |
| Efficient | −15% power, −8 mana, ×0.85 cost |
| Extended | ×1.5 range, +8 mana |
| Swift | −10% power, −12 ticks, +6 mana |
| Empowered | +55% power, +20 mana, +20 ticks |
| Draining | +10% power, +12 mana |
| Overcharged | +95% power, ×1.2 cost, +24 mana, +40 ticks |

`Efficient` cannot be combined with `Empowered` or `Overcharged`, and `Swift`
cannot be combined with `Overcharged`: that is what keeps a cheap, hard-hitting
spell from existing.

## Pyromancy

| Spell | Shape | Effect | Modifiers | Level | Mana | Power | Range | Cooldown |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Fireball (`pyro_fireball`) | Projectile | Damage | Explosive | 1 | 28.0 | 22.3 | 24 | 1.2s |
| Wall of Flames (`pyro_wall_of_flames`) | Wall | Slowing | Extended | 5 | 27.0 | 7.7 | 15 | 5.3s |
| Volcanic Burst (`pyro_volcanic_burst`) | Nova | Damage | Explosive | 8 | 36.0 | 25.5 | 8 | 4.8s |
| Blazing Nova (`pyro_blazing_nova`) | Nova | Damage | Empowered | 14 | 42.0 | 27.3 | 8 | 5.9s |
| Meteor (`pyro_meteor`) | Beam | Damage | Explosive, Overcharged | 20 | 64.8 | 40.4 | 20 | 3.6s |

## Cryomancy

| Spell | Shape | Effect | Modifiers | Level | Mana | Power | Range | Cooldown |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Ice Bolt (`cryo_ice_bolt`) | Projectile | Damage | Piercing | 1 | 24.0 | 16.0 | 29 | 1.1s |
| Snowstorm (`cryo_snowstorm`) | Zone | Slowing | Extended | 6 | 25.0 | 7.6 | 18 | 3.2s |
| Crystal Wall (`cryo_crystal_wall`) | Wall | Rooting | Extended | 10 | 30.0 | 8.6 | 15 | 5.3s |
| Ice Prison (`cryo_ice_prison`) | Zone | Rooting | Extended | 12 | 28.0 | 9.5 | 18 | 3.2s |
| Blizzard (`cryo_blizzard`) | Nova | Slowing | Extended, Swift | 22 | 35.0 | 11.1 | 12 | 3.8s |

## Electromancy

| Spell | Shape | Effect | Modifiers | Level | Mana | Power | Range | Cooldown |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Lightning Bolt (`electro_bolt`) | Beam | Damage | — | 1 | 18.4 | 13.7 | 20 | 1.4s |
| Storm Step (`electro_blink`) | Blink | Teleportation | — | 4 | 13.8 | 0.0 | 12 | 2.5s |
| Chain Lightning (`electro_chain`) | Beam | Damage | Chaining | 10 | 34.4 | 17.1 | 20 | 1.9s |
| Overload (`electro_overload`) | Self | Shielding | Overcharged | 18 | 44.0 | 18.4 | 0 | 4.3s |
| Thunderstorm (`electro_storm`) | Nova | Damage | Chaining, Extended | 26 | 49.3 | 21.0 | 12 | 5.2s |

## Nature Magic

| Spell | Shape | Effect | Modifiers | Level | Mana | Power | Range | Cooldown |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Grasping Roots (`nature_roots`) | Zone | Rooting | — | 1 | 18.0 | 8.5 | 12 | 3.2s |
| Natural Healing (`nature_heal`) | Self | Healing | — | 1 | 8.1 | 6.8 | 0 | 2.1s |
| Thorn Wall (`nature_thorn_wall`) | Wall | Slowing | Extended | 8 | 25.1 | 6.0 | 15 | 5.2s |
| Guardian Spirit (`nature_guardian`) | Self | Shielding | Empowered | 16 | 29.9 | 11.9 | 0 | 3.2s |
| Healing Sanctuary (`nature_sanctuary`) | Zone | Healing | Extended, Efficient | 20 | 13.0 | 6.5 | 18 | 3.2s |

## Shadow Magic

| Spell | Shape | Effect | Modifiers | Level | Mana | Power | Range | Cooldown |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Spectral Blade (`shadow_blade`) | Projectile | Damage | Piercing | 1 | 25.4 | 16.8 | 29 | 1.1s |
| Shadow Step (`shadow_step`) | Blink | Teleportation | Swift | 6 | 19.2 | 0.0 | 12 | 1.9s |
| Vampiric Orb (`shadow_drain`) | Projectile | Draining | Draining | 10 | 28.5 | 14.3 | 24 | 1.1s |
| Umbral Clone (`shadow_clone`) | Nova | Slowing | Draining | 22 | 35.1 | 14.3 | 8 | 4.5s |
| Abyssal Rift (`shadow_rift`) | Beam | Damage | Explosive, Overcharged | 30 | 66.7 | 36.8 | 20 | 3.5s |

## Arcane Magic

| Spell | Shape | Effect | Modifiers | Level | Mana | Power | Range | Cooldown |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Arcane Ray (`arcane_ray`) | Beam | Damage | — | 1 | 20.0 | 14.3 | 20 | 1.4s |
| Arcane Shield (`arcane_shield`) | Self | Shielding | — | 5 | 13.8 | 9.9 | 0 | 2.2s |
| Spatial Distortion (`arcane_distortion`) | Blink | Teleportation | Extended | 12 | 23.0 | 0.0 | 18 | 2.5s |
| Dimensional Burst (`arcane_dimension_burst`) | Nova | Damage | Explosive, Overcharged | 34 | 78.6 | 49.8 | 8 | 7.7s |
| Singularity (`arcane_singularity`) | Zone | Damage | Empowered, Explosive, Chaining | 40 | 72.5 | 34.0 | 12 | 5.3s |

## The schools' identities

| School | Colour | Particle | Sound | Damage type |
| --- | --- | --- | --- | --- |
| Pyromancy | `#FF7A2F` | `minecraft:flame` | `entity.blaze.shoot` | `minecraft:on_fire` |
| Cryomancy | `#7FD8FF` | `minecraft:snowflake` | `block.glass.break` | `minecraft:freeze` |
| Electromancy | `#FFE96B` | `minecraft:electric_spark` | `entity.lightning_bolt.thunder` | `minecraft:lightning_bolt` |
| Nature | `#7CE87C` | `minecraft:composter` | `block.grass.break` | `minecraft:magic` |
| Shadow | `#B07CFF` | `minecraft:squid_ink` | `entity.warden.sonic_boom` | `minecraft:wither` |
| Arcane | `#6FC3FF` | `minecraft:end_rod` | `block.amethyst_block.chime` | `minecraft:indirect_magic` |

Only vanilla identifiers are used, because the authoritative server runs the
effects as vanilla commands; a school's identity is therefore identical for
every player and on every Horizon build.

## Composing your own

The engine accepts any legal combination, and refuses the rest with a reason:

```rust
use arcanum_engine::{Effect, Element, Limits, Modifier, Shape, SpellBuilder};

let spell = SpellBuilder::new(&Limits::default(), "my_spell", "My Spell",
        Element::Cryo, Shape::Nova, Effect::Slow)
    .modifiers(&[Modifier::Extended, Modifier::Swift])
    .required_level(12)
    .build()?;
```

There is no in-game screen for this yet — Horizon has no API for a mod's own
screen (`docs/API_LIMITATIONS.md`) — so the builder is reachable from code and
from a future UI, not from a menu today.
