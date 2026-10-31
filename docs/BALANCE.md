# Balance

Every number a spell derives from is in
[`mixins/arcanum/engine/src/balance.rs`](../mixins/arcanum/engine/src/balance.rs),
and every rule below is enforced by `tests/balance_tests.rs` and
`tests/spell_tests.rs`.

## The three economies

**Mana** — `BASE_MANA` 100, `+4` per level, `+1.5` per Vitality point, regen
`0.35`/tick multiplied by Concentration. Mana is the only cost a cast pays, and
it is paid *before* the effect and refunded if the cast cannot be run at all.

**Cooldown** — derived, not authored: `shape.base_cooldown × (1 + power/60 ×
0.5) + Σ modifier cooldown`, floored at 10 ticks, multiplied by Precision (never
below 65%). A stronger spell therefore locks you out longer *automatically*,
which is what keeps "biggest spell every second" from being the only strategy.

**Resonance** — `+0.35` per point of spell power, decaying `0.08`/tick after
100 ticks without casting, capped at 100. It is the "sustained fight" reward:
stopping to wait loses it.

## How a spell's cost, power, range and cooldown are derived

```
base_cost  = (shape.base_cost  + effect.base_cost)  × element.cost_multiplier
cost       = max(1, (base_cost + Σ modifier.cost_delta) × Π modifier.cost_scale)

power      = (shape.base_power + effect.base_power) × element.power_multiplier
             × Π (1 + modifier.power_bonus)

range      = shape.base_range × Π modifier.range_scale

cooldown   = clamp(round(shape.base_cooldown × (1 + power/60 × 0.5) + Σ modifier.cooldown_delta),
                   10, 1200)  × Precision
```

Damage is `power × 0.55`. A spell's experience reward is `cost × 2`.

## The caps (what makes a modular system safe)

| Cap | Value | Why |
| --- | --- | --- |
| `max_cost` | 140 mana | One cast may not empty a level-50 pool (280) twice over |
| `max_power` | 60 | Damage stays under 33 per cast, before the caster's bonuses |
| `max_range` | 32 blocks | Beyond that, a cast is a sniper rifle |
| `max_modifiers` | 3 | Beyond that, combinations stop being readable |
| `min_cooldown` / `max_cooldown` | 10 / 1200 ticks | No spam, no 5-minute lockouts |
| **`MIN_COST_PER_POWER`** | **0.9** | **The important one** |

The efficiency floor is what a modular spell system usually gets wrong. Without
it, "cheap shape + cheap effect + Efficient" is better than everything else and
the whole builder collapses into one correct answer. With it, a spell must cost
at least 90% of its own power in mana, so power is never free. `Efficient` and
`Empowered`/`Overcharged` are mutually exclusive on top of that, and `Swift`
cannot be stacked with `Overcharged`.

`tests/balance_tests.rs::the_builder_never_returns_a_spell_past_a_cap` tries
every element × shape × effect with every three-modifier combination (over
3,500 combinations), asserts that each accepted spell is inside every cap, and
asserts that the caps reject *some* combinations — a cap that never bites is a
cap that is not doing anything.

## The caps are not the whole story: the caster's bonuses

Progression multiplies the derived numbers, and both multipliers are bounded:

| Source | Effect | Bound |
| --- | --- | --- |
| Power | `+1%` spell power per point | unbounded, but points are finite (98 at level 50) |
| Mastery | `-0.5%` cost per point | never past **-40%** |
| Precision | `-0.5%` cooldown per point | never past **-35%** |
| Vitality | `+1.5` mana per point | — |
| Concentration | `+1%` regen per point | — |
| Elemental mastery | up to **+25%** power in one school | at 100 mastery |
| Specialization | **+15%** power in one school | chosen once, at level 10 |
| Talents | `+5%` to `+22%` power, `-5%` to `-12%` cost | two per school |

Worst case for a level-50 specialist: 60 power × (1 + 0.98 + 0.25 + 0.15 + 0.22)
× 0.55 ≈ 86 damage on a single target, for 140 × 0.35 ≈ 49 mana on a ~1,700-tick
cooldown. That is a boss ability, not a duel opener — which is the intent.

## Levels and experience

`xp_for_next(level) = 100 + 55·level + 8·level²`, capped at level 50. Reaching
50 takes ≈ 380,000 experience; a Fireball is worth 56. Levels pay two stat
points each. A specialization is chosen once, at level 10. Talents unlock at
level 5 (no specialization needed) and level 25 (that school's specialization).

**Spells are earned, not granted**: `PlayerState` learns every spell whose
`required_level` the mage has reached, and nothing else. The starting kit is one
spell per school at level 1, so every school is playable from the first minute.

## What is deliberately *not* balanced yet

- **PvP.** Damage selectors exclude players. A duel mode needs consent, a
  damage scale and a death/loot rule; none of that is invented here.
- **Equipment.** Staffs and robes would add flat bonuses; they are a content mod
  and do not exist yet, so no numbers are claimed for them.
- **Summons.** No mob is spawned by a spell in this version.
- **Ultimates.** Resonance accumulates and the engine can spend it
  (`Resonance::spend_ultimate`), but no ultimate spell is wired to it yet — the
  hook is in the engine, the content is not.

Adding a spell is one line in `registry.rs`; `cargo test` then tells you whether
the line is legal.
