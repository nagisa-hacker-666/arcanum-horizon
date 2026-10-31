# Multiplayer

Arcanum is server-authoritative by construction: the client cannot cast, only
ask. This document says exactly where each decision is made, what travels on
the wire, and what Horizon does and does not allow today.

## The split

```
window (client half)                     server half (authoritative)
────────────────────                     ───────────────────────────
native_hud        ← state snapshot  ←    native_player_tick
native_key        → "cast"          →    native_net_from_client
                  → "select 1"      →      ├─ is the spell in the registry?
                  → "sync"          →      ├─ did this mage learn it?
                                           ├─ is the level high enough?
                                           ├─ is the mana there?
                                           ├─ is the cooldown over?
                                           ├─ is the caster's name usable?
                                           └─ cast::resolve → commands
native_net_from_server ← snapshot     ←    native_run_command (per command)
```

The server keeps a `PlayerState` per UUID: mana, resonance, cooldowns, known
spells, level, experience, stats, mastery, talents, specialization. The client
keeps one thing — the last `StateSnapshot` it was sent — and draws it.

## What a message can say

`ClientMessage` is one line of text (`mixins/arcanum/engine/src/net.rs`):

| Message | Meaning | What the server does |
| --- | --- | --- |
| `cast` | cast the selected spell | full validation, then the commands |
| `select <n>` | move the selection by `n` | clamps and wraps inside the grimoire |
| `sync` | "send my state again" | answers with a snapshot |

Nothing else is accepted: `ClientMessage::decode` returns `None` for anything
else, and `|n| > 100` is refused. A modified client cannot name a spell, a
damage number, a mana cost or a cooldown — it can only say "cast", and the
server looks everything up in its own registry.

`StateSnapshot` is the only server-to-client payload: level, experience, mana,
mana maximum, resonance, cooldown ticks, spell name and school. It is display
data, re-derived on the server from state it owns.

## Why the effects are vanilla commands

Horizon gives a server mod no raycast and no direct damage call, so a cast
resolves into commands the server runs as its console (`native_run_command`,
hook API 21). That keeps the server authoritative and makes the spell visible to
every nearby player with no extra code, but it has two visible consequences:

- the target is the nearest mob within range, not the crosshair (see
  `API_LIMITATIONS.md`);
- damage is dealt as a vanilla damage type per school.

Particles and sounds are commands too, so they are identical for every client:
no player can turn their effects off, and none can see effects another cannot.

## What works today, honestly

| Feature | State |
| --- | --- |
| Casts on a LAN world or a dedicated server | Yes: mod messages (API 40) between the mod's halves |
| Each player's own mana, cooldowns, level, grimoire | Yes: keyed by UUID, saved per player |
| Casts seen by nearby players | Yes: server-emitted particles, sounds and effects |
| Casts on a vanilla server, or from a client without Arcanum | No: the client gets "this server carries no mod messages" in chat |
| Healing and shielding another player | Partly: area heals reach players within the spell's radius (`@a[distance=..R]`), single-target heals reach the caster only |
| Duel damage between two mages | No: damage selectors exclude `minecraft:player`, deliberately — PvP needs a consent and balance design, not just a selector |
| Summons, boss fights, shared grimoires, mage parties | Not implemented |

## Design rules for the next version

1. **New state goes on the server first.** A client field that the server does
   not recompute is a bug waiting to be exploited.
2. **Cap the message rate.** Horizon allows 32 mod messages per player per tick;
   Arcanum sends at most one cast and one select per key press, and one snapshot
   per second per player. Keep it that way: a per-tick sync is 20× the traffic
   for no visible gain.
3. **Never hold a lock across a game call.** The runtime takes a mage out of the
   shared map, does its work, then puts them back, because `native_run_command`
   can re-enter the mod (a damage command can kill a player, which fires
   `native_player_died`). A `Mutex` held across that call deadlocks.
4. **Save on change, not on a timer you hope fires.** A crash between saves
   loses at most the last action.
5. **PvP needs a flag, not a selector.** Add a `duel` concept with mutual
   consent before any player-targeting damage is enabled.
