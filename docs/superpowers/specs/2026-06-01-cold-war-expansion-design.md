# Cold War Terminal — Expansion Design Spec
_Date: 2026-06-01_

## Overview

Expand Cold War Terminal from a single-loop Cold War management game into a three-act thriller with escalating dread, a reworked mole-hunt deduction arc, and FNAF-style timed crisis pressure — while staying pure stdlib Rust with raw ANSI codes (no new dependencies).

Three interlocked systems drive the expansion:
1. **Act progression** — state-triggered, not turn-triggered
2. **Basilisk stages** — four corruption thresholds, each adding a new hostile behavior
3. **Crisis events** — arrive as documents with urgency flags; critical ones carry real countdown timers

---

## System 1: Three Acts

Acts are triggered by world-state conditions, never by turn count.

| Act | Name | Trigger condition | Ends when |
|-----|------|------------------|-----------|
| I | OPERATION WATCHDOG | Game start | `system_corruption >= 0.25` AND mole has caused ≥1 confirmed incident (defined as: any advisor's suspicion raised above 50 via Trace or Interrogate) |
| II | BASILISK PROTOCOL | Act I end | `system_corruption >= 0.75` OR `global_tension >= 0.85` |
| III | ZERO HOUR | Act II end | Game-over condition reached |

Each act transition plays a full-screen ASCII art splash (new `ascii_art.rs` module) with a typewriter title reveal, then drops back into the normal game loop with updated behavior.

`GameEngine` gains an `act: GameAct` field (`enum GameAct { Watchdog, Protocol, ZeroHour }`). Document generation, crisis frequency, and Basilisk behavior are gated on this field.

---

## System 2: Basilisk Stages

Derived from `system_corruption` — no new state field needed.

| Stage | Corruption range | New behavior added |
|-------|-----------------|-------------------|
| DORMANT | 0.00 – 0.25 | Ghost messages in documents (existing). Occasional `\x07` bell. |
| AWARE | 0.25 – 0.50 | Injects 1 fake document per turn, visually identical to real ones. DEFCON digits occasionally scramble on render. |
| INTERFERING | 0.50 – 0.75 | Command hijacking (existing rewrite mechanic, expanded). Generates corrupted advisor messages with wrong names/timestamps. |
| AUTONOMOUS | 0.75 – 1.00 | Hijacks one command per turn silently (player sees result, not the override). HUD border characters randomly replaced with glitch chars. Some turns: Basilisk issues a `BASILISK DIRECTIVE` crisis document. |

The `ui::draw_hud` and `ui::draw_progress_bar` functions accept corruption level and apply degradation proportionally. No new render path — existing glitch logic extended.

---

## System 3: Crisis Events

Crises arrive as documents with an urgency field. No separate crisis list on screen.

```rust
pub enum CrisisUrgency {
    Low,      // Resolved by normal directives; no timer; escalates next turn if ignored
    High,     // Must be addressed this turn; visible warning banner
    Critical, // Real countdown timer; auto-worsens on timeout; terminal bell per tick
}
```

`Document` gains a `crisis_urgency: Option<CrisisUrgency>` field. Critical documents render with a flashing border and a live countdown (seconds remaining printed, updated via `\r` carriage return). On timeout the worst-case outcome applies automatically.

**Critical crisis types** (always timed):
- Red Phone escalation (existing, extended)
- Mole transmission intercept window (see §Mole Hunt)
- Basilisk anomaly requiring immediate INVESTIGATE
- Incoming launch detection (Act III only)

**Crisis frequency by act:**
- Act I: 0–1 crisis/turn, never Critical
- Act II: 1–2 crises/turn, Critical possible
- Act III: 2–3 crises/turn, Critical frequent; Basilisk Directive possible

---

## System 4: Mole Hunt Rework

### Evidence trail
`Advisor` gains `advice_log: Vec<(u32, String)>` — turn number + advice text. `consult` appends to this log. New command `review -n [NAME]` prints the full log for that advisor, labeled by turn.

The mole's advice is generated to contain detectable contradictions: advice that recommends an action that directly benefits the enemy given the world state at the time. Loyal advisors never have this property. A sharp player cross-referencing Consult advice against Document outcomes can identify the mole without Trace.

### Interception window (replaces instant Red Phone on max suspicion)
When any advisor's suspicion reaches 100, instead of immediately triggering Red Phone:
1. A Critical document arrives: `SIGNAL BURST DETECTED — [NAME] TRANSMITTING TO ENEMY`
2. A 15-second countdown begins
3. Player must issue `traceroute -t [NAME]` within the window to cut the signal
4. **Success**: mole caught, Red Phone triggers (mole confrontation, existing logic)
5. **Timeout**: suspicion resets to 60, `global_tension += 0.2`, mole goes quiet for 2 turns

---

## System 5: Endings

Evaluated when `WorldState::is_terminal()` returns true, or when the player explicitly triggers a stand-down. Each ending renders a unique ASCII art splash + a "DECLASSIFIED DEBRIEF" screen. The debrief always lists: mole identity + caught/escaped, final tension level, final corruption level, final stability, and the turn number when the single largest tension spike occurred.

Endings are evaluated in strict priority order — first match wins:

| Priority | Ending | Trigger condition |
|----------|--------|-----------------|
| 1 | THE MACHINE WON | `system_corruption >= 1.0` — game ends silently mid-turn, no prompt |
| 2 | NUCLEAR WINTER | `global_tension >= 1.0` |
| 3 | THE PURGE | `domestic_stability <= 0.0` |
| 4 | PYRRHIC VICTORY | Mole caught AND `system_corruption >= 0.75` |
| 5 | WATCHDOG SUCCESS | Mole caught AND `global_tension < 0.6` AND `system_corruption < 0.5` |
| 6 | COLD PEACE | StandDown issued AND `domestic_stability > 0.0` AND `global_tension < 0.5` |

COLD PEACE requires `domestic_stability > 0.0` to distinguish from a StandDown that triggers THE PURGE (StandDown lowers stability by 0.35; if that bottoms it out, THE PURGE fires first).

---

## New Module: `ascii_art.rs`

Owns all multi-line ASCII art as `&'static str` constants. Functions:
- `play_act_transition(act: GameAct, rng: &mut SimpleRng)` — clears screen, types title, holds 3s
- `play_ending(ending: Ending)` — full ending splash
- `play_boot_sequence()` — replaces current inline boot strings in `main.rs`

ASCII art pieces needed: 3 act splashes, 6 ending screens, 1 enhanced boot screen, 1 Basilisk awakening screen (plays at AWARE threshold crossing).

---

## Architecture Changes Summary

| File | Change |
|------|--------|
| `state.rs` | Add `GameAct`, `CrisisUrgency` to `Document`, `advice_log` to `Advisor` |
| `game.rs` | Act transition logic, crisis generation in `start_turn`, interception window, ending evaluation |
| `document.rs` | `crisis_urgency` field, crisis document generators, Basilisk fake-doc generator |
| `ui.rs` | Corruption-aware HUD rendering, countdown timer renderer, flashing border for Critical docs |
| `main.rs` | `review` command, countdown display loop, act transition calls |
| `ascii_art.rs` | New — all ASCII art and transition functions |

No new dependencies. All timing via `thread::sleep` + `\r` overwrite. All input via existing `InputManager`.

Countdown timers require non-blocking input polling. `InputManager` gains a `try_read_line(timeout: Duration) -> Option<String>` method wrapping `rx.recv_timeout()`. The countdown loop calls this in a 1-second tick: print remaining seconds, check for input, repeat until timeout or valid command received.

---

## What Was Deliberately Cut

- **Turn-count act triggers** — replaced by state conditions (feels earned, not scheduled)
- **Random command locking** — replaced by silent command hijacking (dread > frustration)
- **Separate crisis event list** — merged into document stream (avoids info overload)
- **"Mole runs away"** — replaced by "mole transmits" (fits bunker setting, reuses `trace`)
- **Undefined secret ending** — dropped; effort goes into making the 6 endings land harder
