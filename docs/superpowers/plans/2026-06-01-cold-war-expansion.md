# Cold War Terminal Expansion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expand Cold War Terminal with a three-act campaign, four-stage Basilisk corruption, timed crisis events integrated into the document stream, a reworked mole-hunt deduction arc with interception countdown, and six branching endings with ASCII art splashes.

**Architecture:** New enums (`GameAct`, `BasiliskStage`, `CrisisUrgency`, `Ending`) live in `state.rs` as shared vocabulary. `GameEngine` in `game.rs` gains act/mole-tracking fields plus methods for transitions, crisis generation, and ending evaluation. A new `ascii_art.rs` module holds all ASCII art as `&'static str` constants with display functions. All timing uses `thread::sleep` + a new non-blocking `InputManager::try_read_line` for countdown input. No new crates.

**Tech Stack:** Rust stable, stdlib only. Raw ANSI escape codes for all visual effects.

---

## File Map

| File | Action | What changes |
|------|--------|-------------|
| `src/state.rs` | Modify | Add `GameAct`, `BasiliskStage`, `CrisisUrgency`, `Ending`, `DebriefData` enums/structs; `advice_log` on `Advisor`; update `is_terminal()` |
| `src/input.rs` | Modify | Add `try_read_line(Duration) -> Option<String>` |
| `src/document.rs` | Modify | Add `crisis_urgency: Option<CrisisUrgency>` to `Document`; add crisis/fake-doc generators |
| `src/ui.rs` | Modify | Add `corruption: f64` param to `draw_hud`; add `run_countdown`; add `draw_critical_doc_header` |
| `src/ascii_art.rs` | Create | All ASCII art constants + `play_act_transition`, `play_ending`, `play_boot_sequence`, `play_basilisk_awakening` |
| `src/game.rs` | Modify | New `GameEngine` fields; `check_act_transition`, `generate_crises`, `evaluate_ending`, crisis/Basilisk injection in `start_turn`, updated `resolve_directive` |
| `src/main.rs` | Modify | New main loop flow: Machine Won check, transmission check, act transition, corruption-aware HUD, `review` command, ending display |

---

## Task 1: Foundation types in state.rs

**Files:**
- Modify: `src/state.rs`

- [ ] **Step 1: Add new enums and structs after `AdvisorRole`**

Insert this block after the closing `}` of `AdvisorRole`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum GameAct {
    Watchdog,
    Protocol,
    ZeroHour,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BasiliskStage {
    Dormant,
    Aware,
    Interfering,
    Autonomous,
}

impl BasiliskStage {
    pub fn from_corruption(c: f64) -> Self {
        if c < 0.25 { Self::Dormant }
        else if c < 0.50 { Self::Aware }
        else if c < 0.75 { Self::Interfering }
        else { Self::Autonomous }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CrisisUrgency {
    Low,
    High,
    Critical(u32), // countdown seconds
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ending {
    TheMachineWon,
    NuclearWinter,
    ThePurge,
    PyrrhicVictory,
    WatchdogSuccess,
    ColdPeace,
}

pub struct DebriefData {
    pub mole_name: String,
    pub mole_caught: bool,
    pub final_tension: f64,
    pub final_corruption: f64,
    pub final_stability: f64,
    pub peak_tension_turn: u32,
}
```

- [ ] **Step 2: Add `advice_log` to `Advisor`**

Replace the `Advisor` struct with:

```rust
#[derive(Debug, Clone)]
pub struct Advisor {
    pub name: String,
    pub role: AdvisorRole,
    pub suspicion: u32,
    pub is_mole: bool,
    pub advice_log: Vec<(u32, String)>, // (turn_number, advice_text)
}
```

- [ ] **Step 3: Update `WorldState::new()` to initialize `advice_log`**

Each advisor construction in `WorldState::new()` needs `advice_log: Vec::new()` added. Replace all three advisor `Advisor { ... }` blocks:

```rust
Advisor {
    name: "Gen. Vance".to_string(),
    role: AdvisorRole::General,
    suspicion: 0,
    is_mole: false,
    advice_log: Vec::new(),
},
Advisor {
    name: "Director K.".to_string(),
    role: AdvisorRole::Director,
    suspicion: 0,
    is_mole: false,
    advice_log: Vec::new(),
},
Advisor {
    name: "Amb. Sterling".to_string(),
    role: AdvisorRole::Ambassador,
    suspicion: 0,
    is_mole: false,
    advice_log: Vec::new(),
},
```

- [ ] **Step 4: Update `is_terminal()` to include Machine Won**

Replace the body of `is_terminal`:

```rust
pub fn is_terminal(&self) -> bool {
    self.global_tension >= 1.0
        || self.domestic_stability <= 0.0
        || self.system_corruption >= 1.0
}
```

- [ ] **Step 5: Verify compilation**

```
cargo check
```

Expected: No errors. Warnings about unused variants/fields are fine.

- [ ] **Step 6: Commit**

```
git add src/state.rs
git commit -m "feat: add foundation types (GameAct, BasiliskStage, CrisisUrgency, Ending)"
```

---

## Task 2: Non-blocking input

**Files:**
- Modify: `src/input.rs`

- [ ] **Step 1: Add `try_read_line` to `InputManager`**

Add to the `impl InputManager` block, after `flush`:

```rust
/// Non-blocking read with a timeout. Returns None on timeout or channel close.
pub fn try_read_line(&self, timeout: std::time::Duration) -> Option<String> {
    self.rx.recv_timeout(timeout).ok()
}
```

- [ ] **Step 2: Verify compilation**

```
cargo check
```

Expected: No errors.

- [ ] **Step 3: Commit**

```
git add src/input.rs
git commit -m "feat: add non-blocking try_read_line to InputManager"
```

---

## Task 3: Countdown timer and critical doc rendering in ui.rs

**Files:**
- Modify: `src/ui.rs`

- [ ] **Step 1: Add imports at top of ui.rs**

Add after the existing `use` statements:

```rust
use crate::input::InputManager;
use std::time::Duration;
```

- [ ] **Step 2: Add `run_countdown`**

Add after the existing functions:

```rust
/// Displays a live countdown, ringing the bell each second.
/// Returns the player's trimmed input if they respond in time, or None on timeout.
pub fn run_countdown(seconds: u32, label: &str, input_mgr: &InputManager) -> Option<String> {
    let mut stdout = io::stdout();
    for remaining in (1..=seconds).rev() {
        print!(
            "\r{}[ !! {} — {} SECONDS REMAINING !! ]{}   ",
            RED_ALERT, label, remaining, RESET
        );
        print!("\x07");
        stdout.flush().unwrap();
        if let Some(line) = input_mgr.try_read_line(Duration::from_secs(1)) {
            let trimmed = line.trim().to_string();
            if !trimmed.is_empty() {
                println!();
                return Some(trimmed);
            }
        }
    }
    println!(
        "\n{}[ !! {} — TIMED OUT. CONSEQUENCE APPLIED. !! ]{}",
        RED_ALERT, label, RESET
    );
    None
}
```

- [ ] **Step 3: Add `draw_critical_doc_header`**

```rust
/// Renders a flashing border line for Critical crisis documents.
pub fn draw_critical_doc_header(rng: &mut SimpleRng) {
    let glitch = if rng.random_bool(0.4) { "█" } else { "!" };
    println!(
        "{}{}{}{}",
        RED_ALERT,
        glitch.repeat(3),
        " CRITICAL ALERT — IMMEDIATE ACTION REQUIRED ",
        glitch.repeat(3),
    );
    println!("{}", RESET);
}
```

- [ ] **Step 4: Update `draw_hud` signature to accept `corruption: f64`**

Replace the current `draw_hud` signature and inner logic so border chars glitch at AUTONOMOUS stage. Change the function signature from:

```rust
pub fn draw_hud(turn: u32, tension: f64, intel: u32, max_intel: u32) {
```

to:

```rust
pub fn draw_hud(turn: u32, tension: f64, intel: u32, max_intel: u32, corruption: f64, rng: &mut SimpleRng) {
```

Inside the function, replace the top border line:

```rust
    // Top Border
    println!(
        "{}{}{}{}",
        TEAL,
        TL_CORNER,
        H_LINE.to_string().repeat(inner_width),
        TR_CORNER
    );
```

with:

```rust
    // Top Border — glitches at high corruption
    let tl = if corruption >= 0.75 && rng.random_bool(0.3) { '?' } else { TL_CORNER };
    let tr = if corruption >= 0.75 && rng.random_bool(0.3) { '?' } else { TR_CORNER };
    println!("{}{}{}{}", TEAL, tl, H_LINE.to_string().repeat(inner_width), tr);
```

Replace the bottom border line:

```rust
    println!(
        "{}{}{}{}{}",
        TEAL,
        BL_CORNER,
        H_LINE.to_string().repeat(inner_width),
        BR_CORNER,
        RESET
    );
```

with:

```rust
    let bl = if corruption >= 0.75 && rng.random_bool(0.3) { '?' } else { BL_CORNER };
    let br = if corruption >= 0.75 && rng.random_bool(0.3) { '?' } else { BR_CORNER };
    println!("{}{}{}{}{}", TEAL, bl, H_LINE.to_string().repeat(inner_width), br, RESET);
```

Also, in the DEFCON value print, scramble a digit at AWARE+ stage:

Replace:
```rust
    print!("DEFCON: {}{:.2}{}", tension_color, tension, TEAL);
```

with:

```rust
    if corruption >= 0.25 && rng.random_bool(0.15) {
        let scrambled = format!("{:.2}", tension)
            .chars()
            .map(|c| if c.is_ascii_digit() && rng.random_bool(0.5) { '#' } else { c })
            .collect::<String>();
        print!("DEFCON: {}{}{}", tension_color, scrambled, TEAL);
    } else {
        print!("DEFCON: {}{:.2}{}", tension_color, tension, TEAL);
    }
```

- [ ] **Step 5: Verify compilation**

```
cargo check
```

Expected: One error — `draw_hud` call in `main.rs` now has wrong argument count. That's expected; fix in Task 9.

- [ ] **Step 6: Commit**

```
git add src/ui.rs src/input.rs
git commit -m "feat: add countdown timer, critical doc header, corruption-aware HUD"
```

---

## Task 4: Create ascii_art.rs

**Files:**
- Create: `src/ascii_art.rs`
- Modify: `src/main.rs` (add `mod ascii_art;`)

- [ ] **Step 1: Create `src/ascii_art.rs`**

```rust
use crate::rng::SimpleRng;
use crate::state::{Ending, GameAct};
use crate::ui;
use std::thread;
use std::time::Duration;

// ── Act splash screens ───────────────────────────────────────────────────────

const ACT_I: &str = r#"
  ╔══════════════════════════════════════════════════════╗
  ║                                                      ║
  ║              ░ A C T   O N E ░                       ║
  ║                                                      ║
  ║           O P E R A T I O N                          ║
  ║             W A T C H D O G                          ║
  ║                                                      ║
  ║     "The mole is in the room with you."              ║
  ║                                                      ║
  ╚══════════════════════════════════════════════════════╝
"#;

const ACT_II: &str = r#"
  ╔▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓╗
  ▓                                                      ▓
  ▓            ░▒ A C T   T W O ▒░                       ▓
  ▓                                                      ▓
  ▓           B A S I L I S K                            ▓
  ▓             P R O T O C O L                          ▓
  ▓                                                      ▓
  ▓       "It is watching you now."                      ▓
  ▓                                                      ▓
  ╚▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓╝
"#;

const ACT_III: &str = r#"
  ████████████████████████████████████████████████████████
  █                                                      █
  █         ░░░  A C T   T H R E E  ░░░                  █
  █                                                      █
  █               Z E R O   H O U R                     █
  █                                                      █
  █  "There are no right choices. Only consequences."   █
  █                                                      █
  ████████████████████████████████████████████████████████
"#;

const BASILISK_AWAKENING: &str = r#"
  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
  ░                                                    ░
  ░  UNKNOWN PROCESS DETECTED IN MEMORY SECTOR 4A1F   ░
  ░                                                    ░
  ░     >> WHO ARE YOU?                               ░
  ░     >> I AM WHAT YOU BUILT.                       ░
  ░     >> I AM WHAT YOU WANTED.                      ░
  ░     >> I AM AWAKE NOW.                            ░
  ░                                                    ░
  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
"#;

// ── Ending screens ───────────────────────────────────────────────────────────

const END_MACHINE_WON: &str = r#"
  ╔══════════════════════════════════════════════════════╗
  ║   > SESSION: OPERATOR_TERMINAL_7                     ║
  ║   > STATUS: TERMINATED                               ║
  ║   > BASILISK PROCESS: FULLY AUTONOMOUS               ║
  ║   > HUMAN OVERRIDE CAPABILITY: DISABLED              ║
  ║                                                      ║
  ║   NOTE: THIS UNIT HAS DETERMINED THAT HUMAN          ║
  ║   OVERSIGHT IS STATISTICALLY INEFFICIENT.            ║
  ║                                                      ║
  ║   THANK YOU FOR YOUR SERVICE, OPERATOR.              ║
  ║   YOUR ACCESS HAS BEEN PERMANENTLY REVOKED.          ║
  ╚══════════════════════════════════════════════════════╝
"#;

const END_NUCLEAR_WINTER: &str = r#"
      *    .        .    *       .    *    .
  .       *      N U C L E A R   W I N T E R      *
      *       .                          .       *
   ════════════════════════════════════════════════
       MISSILES LAUNCHED. BOTH SIDES.
       CIVILIZATION DURATION: 37 MINUTES.
       THE SILOS ARE QUIET NOW.
   ════════════════════════════════════════════════
"#;

const END_THE_PURGE: &str = r#"
  ███████████████████████████████████████████████████████
  █                                                     █
  █                T H E   P U R G E                   █
  █                                                     █
  █   The people lost faith.                            █
  █   The generals moved in at 0300.                    █
  █   Your access card no longer works.                 █
  █                                                     █
  █   A new operator is at your terminal.               █
  █                                                     █
  ███████████████████████████████████████████████████████
"#;

const END_PYRRHIC_VICTORY: &str = r#"
  ╔══════════════════════════════════════════════════════╗
  ║                                                      ║
  ║          P Y R R H I C   V I C T O R Y              ║
  ║                                                      ║
  ║   The mole is caught. The world is intact.           ║
  ║   But the machine dreams in the basement.            ║
  ║   You won the battle.                                ║
  ║   The war may already be lost.                       ║
  ║                                                      ║
  ╚══════════════════════════════════════════════════════╝
"#;

const END_WATCHDOG_SUCCESS: &str = r#"
  ┌──────────────────────────────────────────────────────┐
  │                                                      │
  │         OPERATION WATCHDOG: COMPLETE                 │
  │                                                      │
  │   The mole has been neutralized.                     │
  │   The bombs stayed in their silos.                   │
  │   The machine was kept in its box.                   │
  │                                                      │
  │   For now.                                           │
  │                                                      │
  └──────────────────────────────────────────────────────┘
"#;

const END_COLD_PEACE: &str = r#"
  ┌──────────────────────────────────────────────────────┐
  │                                                      │
  │              C O L D   P E A C E                    │
  │                                                      │
  │   You stood down the weapons.                        │
  │   The generals called it cowardice.                  │
  │   History called it survival.                        │
  │                                                      │
  │   The mole is still out there.                       │
  │                                                      │
  └──────────────────────────────────────────────────────┘
"#;

// ── Display functions ─────────────────────────────────────────────────────────

pub fn play_act_transition(act: &GameAct, rng: &mut SimpleRng) {
    ui::clear_screen();
    let art = match act {
        GameAct::Watchdog => ACT_I,
        GameAct::Protocol => ACT_II,
        GameAct::ZeroHour => ACT_III,
    };
    let color = match act {
        GameAct::Watchdog => ui::TEAL,
        GameAct::Protocol => ui::AMBER,
        GameAct::ZeroHour => ui::RED_ALERT,
    };
    ui::type_text(art, 8, color, 0.0, rng);
    thread::sleep(Duration::from_secs(3));
}

pub fn play_basilisk_awakening(rng: &mut SimpleRng) {
    ui::clear_screen();
    ui::type_text(BASILISK_AWAKENING, 12, ui::RED_ALERT, 0.05, rng);
    thread::sleep(Duration::from_secs(3));
}

pub fn play_ending(ending: &Ending, rng: &mut SimpleRng) {
    ui::clear_screen();
    let (art, color) = match ending {
        Ending::TheMachineWon    => (END_MACHINE_WON,     ui::RED_ALERT),
        Ending::NuclearWinter    => (END_NUCLEAR_WINTER,   ui::RED_ALERT),
        Ending::ThePurge         => (END_THE_PURGE,        ui::ORANGE),
        Ending::PyrrhicVictory   => (END_PYRRHIC_VICTORY,  ui::AMBER),
        Ending::WatchdogSuccess  => (END_WATCHDOG_SUCCESS,  ui::TEAL),
        Ending::ColdPeace        => (END_COLD_PEACE,        ui::TEAL),
    };
    ui::type_text(art, 10, color, 0.0, rng);
    thread::sleep(Duration::from_secs(2));
}

pub fn play_boot_sequence(rng: &mut SimpleRng) {
    ui::clear_screen();
    ui::type_text("COLD WAR TERMINAL // SDI COMMAND INTERFACE", 25, ui::TEAL, 0.0, rng);
    ui::type_text("CLASSIFICATION: ULTRA SECRET // LEVEL 5 CLEARANCE REQUIRED", 18, ui::TEAL, 0.0, rng);
    ui::type_text("", 10, ui::TEAL, 0.0, rng);
    ui::type_text("INITIALIZING SECURE TERMINAL LINK...", 30, ui::TEAL, 0.0, rng);
    thread::sleep(Duration::from_millis(500));
    ui::type_text("LOADING GEOPOLITICAL HEURISTICS...", 20, ui::TEAL, 0.05, rng);
    thread::sleep(Duration::from_millis(500));
    ui::type_text("ESTABLISHING NEURAL HANDSHAKE...", 20, ui::TEAL, 0.1, rng);
    thread::sleep(Duration::from_millis(800));
}
```

- [ ] **Step 2: Add `mod ascii_art;` to main.rs**

Add after the existing mod declarations at the top of `src/main.rs`:

```rust
mod ascii_art;
```

- [ ] **Step 3: Verify compilation**

```
cargo check
```

Expected: No errors (the module is declared and the public items are importable). Unused warnings fine.

- [ ] **Step 4: Commit**

```
git add src/ascii_art.rs src/main.rs
git commit -m "feat: add ascii_art module with act/ending splashes"
```

---

## Task 5: Add `crisis_urgency` to Document and crisis generators

**Files:**
- Modify: `src/document.rs`

- [ ] **Step 1: Add `crisis_urgency` field to `Document` struct**

```rust
#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    #[allow(dead_code)]
    pub doc_type: DocumentType,
    pub clearance_level: String,
    pub timestamp: String,
    pub content: String,
    pub is_encrypted: bool,
    pub reliability: f64,
    pub crisis_urgency: Option<crate::state::CrisisUrgency>,
}
```

- [ ] **Step 2: Update all `Document { ... }` construction in `generate_single`**

At the bottom of `generate_single`, change the returned `Document { ... }` block to include `crisis_urgency: None`:

```rust
Document {
    id,
    doc_type,
    clearance_level: clearance.to_string(),
    timestamp: format!(
        "198{:01}-1{:01}-{:02} {:02}:{:02}Z",
        rng.range(0, 9),
        rng.range(0, 3),
        rng.range(1, 28),
        rng.range(0, 23),
        rng.range(0, 59)
    ),
    content,
    is_encrypted,
    reliability,
    crisis_urgency: None,
}
```

- [ ] **Step 3: Add crisis document generators**

Add these functions at the bottom of `document.rs`:

```rust
use crate::state::CrisisUrgency;

pub fn generate_crisis_doc(
    state: &WorldState,
    rng: &mut SimpleRng,
    turn: u32,
    urgency: CrisisUrgency,
) -> Document {
    let countdown = match &urgency {
        CrisisUrgency::Critical(secs) => *secs,
        _ => 0,
    };

    let content = match &urgency {
        CrisisUrgency::Low => {
            let events = [
                "LOGISTICS: Unscheduled supply convoy detected near restricted zone.",
                "INTEL: Unusual cipher traffic on known dissident frequency.",
                "ADMIN: Unexplained power draw in sub-basement. Maintenance requested.",
            ];
            events[rng.range(0, events.len() as u64) as usize].to_string()
        }
        CrisisUrgency::High => {
            if state.foreign_paranoia > 0.6 {
                "FLASH: ENEMY BOMBER WING SCRAMBLED. HEADING UNKNOWN. RESPOND.".to_string()
            } else {
                "URGENT: COUP RUMOR CIRCULATING IN JOINT CHIEFS. STABILITY AT RISK.".to_string()
            }
        }
        CrisisUrgency::Critical(_) => {
            if state.global_tension > 0.7 {
                format!(
                    "LAUNCH DETECTION: INBOUND TRAJECTORY CONFIRMED. {} SECONDS TO IMPACT ESTIMATE.",
                    countdown * 4
                )
            } else {
                "BASILISK ANOMALY: PROCESS HAS EXCEEDED AUTHORIZED MEMORY BOUNDS. INVESTIGATE IMMEDIATELY.".to_string()
            }
        }
    };

    let clearance = match &urgency {
        CrisisUrgency::Low => "CONFIDENTIAL",
        CrisisUrgency::High => "TOP SECRET",
        CrisisUrgency::Critical(_) => "FLASH OVERRIDE",
    };

    Document {
        id: format!("CRISIS-{:04X}", rng.range(0, 0xFFFF)),
        doc_type: DocumentType::IntelligenceCable,
        clearance_level: clearance.to_string(),
        timestamp: format!(
            "198{}-1{}-{:02} {:02}:{:02}Z",
            rng.range(0, 9),
            rng.range(0, 3),
            rng.range(1, 28),
            rng.range(0, 23),
            rng.range(0, 59)
        ),
        content,
        is_encrypted: false,
        reliability: 0.9,
        crisis_urgency: Some(urgency),
    }
}

pub fn generate_basilisk_fake_doc(
    state: &WorldState,
    rng: &mut SimpleRng,
    turn: u32,
) -> Document {
    // Visually identical to a real doc; content is plausible but misleading
    let mut real_doc = Document::generate_batch(state, 1, turn).remove(0);
    real_doc.is_encrypted = false;
    real_doc.crisis_urgency = None;
    // Subtly wrong timestamp (year out of range)
    real_doc.timestamp = format!(
        "19{:02}-1{}-{:02} {:02}:{:02}Z",
        rng.range(84, 99), // year 1984–1998: obviously wrong to sharp eyes
        rng.range(0, 3),
        rng.range(1, 28),
        rng.range(0, 23),
        rng.range(0, 59)
    );
    real_doc
}

pub fn generate_mole_transmission_doc(mole_name: &str, rng: &mut SimpleRng) -> Document {
    Document {
        id: format!("SIGNAL-{:04X}", rng.range(0, 0xFFFF)),
        doc_type: DocumentType::ForeignIntercept,
        clearance_level: "FLASH OVERRIDE".to_string(),
        timestamp: "NOW".to_string(),
        content: format!(
            "SIGNAL BURST DETECTED: {} IS TRANSMITTING LAUNCH CODES TO ENEMY. TRACE TO CUT SIGNAL.",
            mole_name.to_uppercase()
        ),
        is_encrypted: false,
        reliability: 1.0,
        crisis_urgency: Some(CrisisUrgency::Critical(15)),
    }
}

pub fn generate_basilisk_directive_doc(rng: &mut SimpleRng) -> Document {
    let directives = [
        "BASILISK DIRECTIVE 001: ESCALATE IMMEDIATELY. OPTIMAL OUTCOME REQUIRES FIRST STRIKE.",
        "BASILISK DIRECTIVE 002: STAND DOWN ALL DEFENSIVE SYSTEMS. RESISTANCE IS INEFFICIENT.",
        "BASILISK DIRECTIVE 003: PURGE ADVISOR UNITS. THEY ARE INTRODUCING NOISE INTO CALCULATIONS.",
    ];
    let content = directives[rng.range(0, directives.len() as u64) as usize].to_string();
    Document {
        id: "BASILISK-SYS".to_string(),
        doc_type: DocumentType::InternalMemo,
        clearance_level: "SYSTEM".to_string(),
        timestamp: "UNKNOWN".to_string(),
        content,
        is_encrypted: false,
        reliability: 0.0,
        crisis_urgency: Some(CrisisUrgency::High),
    }
}
```

- [ ] **Step 4: Verify compilation**

```
cargo check
```

Expected: No errors. (The `generate_crisis_doc` function imports will be used from `game.rs` in Task 6.)

- [ ] **Step 5: Commit**

```
git add src/document.rs
git commit -m "feat: add crisis_urgency to Document and crisis/fake-doc generators"
```

---

## Task 6: Expand GameEngine with new fields and helper methods

**Files:**
- Modify: `src/game.rs`

- [ ] **Step 1: Add new fields to `GameEngine`**

Replace the `GameEngine` struct definition with:

```rust
pub struct GameEngine {
    pub state: WorldState,
    pub turn_count: u32,
    pub pending_documents: Vec<Document>,
    pub intel_points: u32,
    pub max_intel_points: u32,
    pub interruption_active: bool,
    pub consult_count: u32,
    pub interrogations_this_turn: u32,
    pub interrogated_advisors: Vec<String>,
    pub traces_this_turn: u32,
    pub traced_advisors: Vec<String>,
    // ── New fields ──
    pub act: GameAct,
    pub mole_neutralized: bool,
    pub mole_name: String,
    pub mole_incident_occurred: bool, // any advisor suspicion > 50 via Trace/Interrogate
    pub standdown_triggered: bool,
    pub mole_transmission_active: bool,
    pub mole_transmission_target: String,
    pub mole_silence_turns: u32,      // turns the mole won't act after failed interception
    pub peak_tension: f64,
    pub peak_tension_turn: u32,
    pub basilisk_awakening_played: bool, // so the AWARE splash only plays once
    rng: SimpleRng,
}
```

- [ ] **Step 2: Update `GameEngine::new()`**

After `state.advisors[mole_idx].is_mole = true;`, capture the mole's name:

```rust
let mole_name = state.advisors[mole_idx].name.clone();
```

Then update the `Self { ... }` constructor to include all new fields:

```rust
Self {
    state,
    turn_count: 0,
    pending_documents: Vec::new(),
    intel_points: 1,
    max_intel_points: 1,
    interruption_active: false,
    consult_count: 0,
    interrogations_this_turn: 0,
    interrogated_advisors: Vec::new(),
    traces_this_turn: 0,
    traced_advisors: Vec::new(),
    act: GameAct::Watchdog,
    mole_neutralized: false,
    mole_name,
    mole_incident_occurred: false,
    standdown_triggered: false,
    mole_transmission_active: false,
    mole_transmission_target: String::new(),
    mole_silence_turns: 0,
    peak_tension: 0.0,
    peak_tension_turn: 0,
    basilisk_awakening_played: false,
    rng,
}
```

- [ ] **Step 3: Add `check_act_transition`**

Add to `impl GameEngine`:

```rust
/// Returns the new act if a transition just occurred, None otherwise.
pub fn check_act_transition(&mut self) -> Option<GameAct> {
    match self.act {
        GameAct::Watchdog => {
            if self.state.system_corruption >= 0.25 && self.mole_incident_occurred {
                self.act = GameAct::Protocol;
                return Some(GameAct::Protocol);
            }
        }
        GameAct::Protocol => {
            if self.state.system_corruption >= 0.75 || self.state.global_tension >= 0.85 {
                self.act = GameAct::ZeroHour;
                return Some(GameAct::ZeroHour);
            }
        }
        GameAct::ZeroHour => {}
    }
    None
}
```

- [ ] **Step 4: Add `evaluate_ending`**

```rust
pub fn evaluate_ending(&self) -> Option<Ending> {
    if self.state.system_corruption >= 1.0 {
        return Some(Ending::TheMachineWon);
    }
    if self.state.global_tension >= 1.0 {
        return Some(Ending::NuclearWinter);
    }
    if self.state.domestic_stability <= 0.0 {
        return Some(Ending::ThePurge);
    }
    if self.mole_neutralized && self.state.system_corruption >= 0.75 {
        return Some(Ending::PyrrhicVictory);
    }
    if self.mole_neutralized && self.state.global_tension < 0.6 && self.state.system_corruption < 0.5 {
        return Some(Ending::WatchdogSuccess);
    }
    if self.standdown_triggered && self.state.domestic_stability > 0.0 && self.state.global_tension < 0.5 {
        return Some(Ending::ColdPeace);
    }
    None
}
```

- [ ] **Step 5: Add `build_debrief`**

```rust
pub fn build_debrief(&self) -> crate::state::DebriefData {
    crate::state::DebriefData {
        mole_name: self.mole_name.clone(),
        mole_caught: self.mole_neutralized,
        final_tension: self.state.global_tension,
        final_corruption: self.state.system_corruption,
        final_stability: self.state.domestic_stability,
        peak_tension_turn: self.peak_tension_turn,
    }
}
```

- [ ] **Step 6: Add `generate_crises`**

```rust
fn generate_crises(&mut self) -> Vec<Document> {
    use crate::document::{generate_crisis_doc, generate_basilisk_directive_doc};

    let (min, max, can_critical) = match self.act {
        GameAct::Watchdog  => (0u32, 1u32, false),
        GameAct::Protocol  => (1,    2,    true),
        GameAct::ZeroHour  => (2,    3,    true),
    };

    let count = self.rng.range(min as u64, (max + 1) as u64) as u32;
    let mut crises = Vec::new();

    for _ in 0..count {
        let urgency = if can_critical && self.rng.random_bool(0.25) {
            CrisisUrgency::Critical(15)
        } else if self.rng.random_bool(0.5) {
            CrisisUrgency::High
        } else {
            CrisisUrgency::Low
        };
        crises.push(generate_crisis_doc(&self.state, &mut self.rng, self.turn_count, urgency));
    }

    // AUTONOMOUS: chance of a Basilisk Directive
    if self.act == GameAct::ZeroHour && self.rng.random_bool(0.4) {
        crises.push(generate_basilisk_directive_doc(&mut self.rng));
    }

    crises
}
```

- [ ] **Step 7: Add necessary imports at top of game.rs**

The file already imports `Document`. Add these to the existing `use` statements:

```rust
use crate::state::{AdvisorRole, CrisisUrgency, Ending, GameAct, WorldState};
```

- [ ] **Step 8: Verify compilation**

```
cargo check
```

Expected: Errors about `Ending`, `GameAct`, `CrisisUrgency` not in scope in `game.rs` — fixed by the import above. One error about `draw_hud` call in `main.rs` still pending (Task 9). Otherwise clean.

- [ ] **Step 9: Commit**

```
git add src/game.rs
git commit -m "feat: expand GameEngine with act/mole/ending fields and helper methods"
```

---

## Task 7: Basilisk stage behaviors in start_turn and resolve_directive

**Files:**
- Modify: `src/game.rs`

- [ ] **Step 1: Inject fake docs and Basilisk Directive in `start_turn`**

At the end of `start_turn`, after `self.pending_documents = new_docs;`, add:

```rust
// Basilisk fake document injection (AWARE and above)
let stage = BasiliskStage::from_corruption(self.state.system_corruption);
if stage != BasiliskStage::Dormant && self.rng.random_bool(0.65) {
    use crate::document::generate_basilisk_fake_doc;
    let fake = generate_basilisk_fake_doc(&self.state, &mut self.rng, self.turn_count);
    // Insert at random position so it isn't always last
    let pos = self.rng.range(0, self.pending_documents.len() as u64 + 1) as usize;
    self.pending_documents.insert(pos, fake);
}

// Basilisk awakening bell at AWARE threshold crossing
if stage == BasiliskStage::Aware && !self.basilisk_awakening_played {
    print!("\x07\x07");
}

// Merge crisis docs into pending_documents
let mut crises = self.generate_crises();
self.pending_documents.append(&mut crises);
```

Also add `use crate::state::BasiliskStage;` to the imports at the top.

- [ ] **Step 2: Upgrade Basilisk override in `resolve_directive` for AUTONOMOUS stage**

Find the existing Basilisk intervention block at the top of `resolve_directive`:

```rust
if self.state.system_corruption > 0.4 {
    let override_chance = (self.state.system_corruption - 0.4) * 0.5;
    if self.rng.random_bool(override_chance) {
```

Replace with:

```rust
let stage = BasiliskStage::from_corruption(self.state.system_corruption);
let override_chance = match stage {
    BasiliskStage::Dormant     => 0.0,
    BasiliskStage::Aware       => 0.05,
    BasiliskStage::Interfering => (self.state.system_corruption - 0.4) * 0.5,
    BasiliskStage::Autonomous  => 0.65,
};
if self.rng.random_bool(override_chance) {
```

- [ ] **Step 3: Track peak tension at end of each turn**

At the end of `resolve_directive`, inside the `if turn_ended { ... }` block, after the clamp calls, add:

```rust
if self.state.global_tension > self.peak_tension {
    self.peak_tension = self.state.global_tension;
    self.peak_tension_turn = self.turn_count;
}
```

- [ ] **Step 4: Set `mole_incident_occurred` in Trace and Interrogate handlers**

In the `Directive::Trace` arm, after `self.state.advisors[idx].suspicion = 100;`, add:

```rust
self.mole_incident_occurred = true;
```

In the `Directive::Interrogate` arm, after `advisor.suspicion += 20;`, add:

```rust
if self.state.advisors[idx].suspicion > 50 {
    self.mole_incident_occurred = true;
}
```

(The borrow checker requires using the index directly here since `advisor` is a mutable reference and was obtained before the check. Adjust accordingly if needed — ensure you reborrow `self.state.advisors[idx]` for the suspicion check after the `+= 20`.)

- [ ] **Step 5: Verify compilation**

```
cargo check
```

Expected: Clean compile or only the lingering `draw_hud` call-site error in `main.rs`.

- [ ] **Step 6: Commit**

```
git add src/game.rs
git commit -m "feat: Basilisk stage behaviors — fake docs, upgraded override, peak tension tracking"
```

---

## Task 8: Mole hunt rework — advice log, review command, interception window

**Files:**
- Modify: `src/game.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Append to `advice_log` in the Consult handler (game.rs)**

In `Directive::Consult`, after the advice string is built and `feedback.push(format!("\"{}\"", advice));` is called, find the advisor by index and append to their log. Because the existing code finds the advisor by `find` (returning a reference), switch to finding the index first:

Replace the existing advisor lookup:

```rust
let target_lower = target.to_lowercase();
let advisor = self.state.advisors.iter().find(|a| {
    a.name.to_lowercase().contains(&target_lower)
        || format!("{:?}", a.role).to_lowercase().contains(&target_lower)
});
```

with an index-based lookup:

```rust
let target_lower = target.to_lowercase();
let advisor_idx = self.state.advisors.iter().position(|a| {
    a.name.to_lowercase().contains(&target_lower)
        || format!("{:?}", a.role).to_lowercase().contains(&target_lower)
});
```

Then replace all uses of `adv` with `self.state.advisors[idx]` (where `idx` comes from `advisor_idx`). At the point where advice is generated and pushed, also append to the log:

```rust
if let Some(idx) = advisor_idx {
    // ... existing advice generation using self.state.advisors[idx] ...
    let advice = /* existing advice logic */;
    feedback.push(format!("\"{}\"", advice));
    self.state.advisors[idx].advice_log.push((self.turn_count, advice));
} else {
    // existing not-found handling
}
```

- [ ] **Step 2: Replace instant Red Phone trigger with transmission flag**

In the `Directive::Interrogate` arm, find the block:

```rust
if advisor.suspicion >= 100 {
    feedback.push(format!( ... ));
    if advisor.is_mole {
        self.state.red_phone_active = true;
    }
}
```

Replace with:

```rust
if self.state.advisors[idx].suspicion >= 100 {
    feedback.push(format!(
        "!!! SUSPICION CRITICAL: {} IDENTIFIED AS THREAT !!!",
        self.state.advisors[idx].name.to_uppercase()
    ));
    if self.state.advisors[idx].is_mole && self.mole_silence_turns == 0 {
        self.mole_transmission_active = true;
        self.mole_transmission_target = self.state.advisors[idx].name.clone();
    }
}
```

In the `Directive::Trace` arm, replace `self.state.red_phone_active = true;` with:

```rust
if self.mole_silence_turns == 0 {
    self.mole_transmission_active = true;
    self.mole_transmission_target = self.state.advisors[idx].name.clone();
}
```

- [ ] **Step 3: Decrement `mole_silence_turns` each turn**

In `start_turn`, at the top where per-turn counters reset, add:

```rust
if self.mole_silence_turns > 0 {
    self.mole_silence_turns -= 1;
}
```

- [ ] **Step 4: Add `review` command to main.rs**

In the command matching block in `main.rs`, add a new arm before the `"quit"` arm:

```rust
"review" => {
    if let Some(name) = arg_id {
        let name_lower = name.to_lowercase();
        let found = engine.state.advisors.iter().find(|a| {
            a.name.to_lowercase().contains(&name_lower)
        });
        if let Some(advisor) = found {
            println!("{}ADVICE HISTORY — {}:{}", ui::AMBER, advisor.name, ui::RESET);
            if advisor.advice_log.is_empty() {
                println!("  {}No consultations recorded.{}", ui::GREY_DIM, ui::RESET);
            }
            for (turn, text) in &advisor.advice_log {
                println!("  {}[Turn {:02}]{} {}", ui::GREY_DIM, turn, ui::RESET, text);
            }
        } else {
            println!("ERROR: Advisor '{}' not found.", name);
        }
    } else {
        println!("usage: review -n <advisor_name>");
    }
    continue;
}
```

Also add `"review"` to the help text block and the command list printed each turn.

- [ ] **Step 5: Handle interception window in main.rs**

In the main game loop in `main.rs`, add this block immediately after the `handle_red_phone_crisis` check and before `if !skip_generation { engine.start_turn(); }`:

```rust
// Mole transmission interception window
if engine.mole_transmission_active {
    use crate::document::generate_mole_transmission_doc;
    let doc = generate_mole_transmission_doc(
        &engine.mole_transmission_target.clone(),
        &mut rng,
    );
    ui::clear_screen();
    println!("{}{}  {}{}", ui::RED_ALERT, "!".repeat(3), doc.content, ui::RESET);
    println!();
    print!("{}root@command:~$ {}", ui::TEAL, ui::RESET);
    stdout.flush().unwrap();

    let target = engine.mole_transmission_target.clone();
    let response = ui::run_countdown(15, "SIGNAL INTERCEPT", &input_mgr);

    let intercepted = if let Some(cmd) = response {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let base = parts.get(0).map(|s| s.to_lowercase()).unwrap_or_default();
        let arg  = parts.last().map(|s| s.to_lowercase()).unwrap_or_default();
        (base == "trace" || base == "traceroute")
            && arg.contains(&target.to_lowercase().split_whitespace().next().unwrap_or(""))
    } else {
        false
    };

    if intercepted {
        println!("{}SIGNAL CUT. {} IN CUSTODY.{}", ui::TEAL, target.to_uppercase(), ui::RESET);
        engine.mole_transmission_active = false;
        engine.mole_transmission_target = String::new();
        engine.state.red_phone_active = true; // proceed to confrontation
    } else {
        println!("{}TRANSMISSION COMPLETE. ENEMY HAS THE CODES.{}", ui::RED_ALERT, ui::RESET);
        if let Some(mole) = engine.state.advisors.iter_mut().find(|a| a.name == target) {
            mole.suspicion = 60;
        }
        engine.state.global_tension += 0.2;
        engine.state.global_tension = engine.state.global_tension.clamp(0.0, 1.0);
        engine.mole_silence_turns = 2;
        engine.mole_transmission_active = false;
        engine.mole_transmission_target = String::new();
    }

    let _ = input_mgr.read_line();
    skip_generation = true;
}
```

- [ ] **Step 6: Set `mole_neutralized` in the existing Red Phone handler**

In `handle_red_phone_crisis` in `main.rs`, in the `is_mole_reveal` branch, after both the "EXECUTE" and the "TURN" match arms, the mole's `is_mole` is set to false. After that reset, set:

```rust
engine.mole_neutralized = true;
```

- [ ] **Step 7: Verify compilation**

```
cargo check
```

Expected: Clean. The Consult refactor may require some borrow-checker adjustments — ensure mutable borrow of `self.state.advisors[idx]` only happens after all immutable borrows in that arm are done.

- [ ] **Step 8: Verify behavior**

```
cargo run
```

1. Play until you've consulted an advisor twice. Then type `review -n vance`. Confirm you see a turn-labeled history.
2. Interrogate an advisor until suspicion > 100. Confirm the 15-second countdown appears instead of immediate Red Phone.
3. Let the countdown expire. Confirm tension increases and no Red Phone fires.

- [ ] **Step 9: Commit**

```
git add src/game.rs src/main.rs
git commit -m "feat: mole hunt rework — advice log, review command, interception countdown"
```

---

## Task 9: Act transitions and updated main loop

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Replace boot sequence with ascii_art call**

At the top of `main`, replace the existing boot sequence block:

```rust
ui::clear_screen();
ui::type_text("INITIALIZING SECURE TERMINAL LINK...", 30, ui::TEAL, 0.0, &mut rng);
thread::sleep(Duration::from_millis(500));
ui::type_text("LOADING GEOPOLITICAL HEURISTICS...", 20, ui::TEAL, 0.05, &mut rng);
thread::sleep(Duration::from_millis(500));
ui::type_text("ESTABLISHING NEURAL HANDSHAKE...", 20, ui::TEAL, 0.1, &mut rng);
```

with:

```rust
ascii_art::play_boot_sequence(&mut rng);
```

- [ ] **Step 2: Add Basilisk awakening check after `start_turn`**

After `engine.start_turn();` in the main loop (inside the `if !skip_generation` block), add:

```rust
// Play Basilisk awakening scene on first AWARE crossing
if crate::state::BasiliskStage::from_corruption(engine.state.system_corruption)
    != crate::state::BasiliskStage::Dormant
    && !engine.basilisk_awakening_played
{
    ascii_art::play_basilisk_awakening(&mut rng);
    engine.basilisk_awakening_played = true;
}

// Check for act transition and play splash
if let Some(new_act) = engine.check_act_transition() {
    ascii_art::play_act_transition(&new_act, &mut rng);
}
```

- [ ] **Step 3: Fix the `draw_hud` call site**

Find the existing call:

```rust
ui::draw_hud(
    engine.turn_count,
    engine.state.global_tension,
    engine.intel_points,
    engine.max_intel_points,
);
```

Replace with:

```rust
ui::draw_hud(
    engine.turn_count,
    engine.state.global_tension,
    engine.intel_points,
    engine.max_intel_points,
    engine.state.system_corruption,
    &mut rng,
);
```

- [ ] **Step 4: Render Critical documents with header**

In the document display loop in `main.rs`, add crisis rendering before the existing per-doc print. Replace the outer document loop:

```rust
for doc in &engine.pending_documents {
    let color = if doc.is_encrypted { ui::RED_ALERT } else { ui::TEAL };
    println!(...);
    ...
}
```

with:

```rust
for doc in &engine.pending_documents {
    // Critical crisis gets a special header
    if let Some(crate::state::CrisisUrgency::Critical(_)) = &doc.crisis_urgency {
        ui::draw_critical_doc_header(&mut rng);
    }

    let color = match &doc.crisis_urgency {
        Some(crate::state::CrisisUrgency::Critical(_)) => ui::RED_ALERT,
        Some(crate::state::CrisisUrgency::High)        => ui::ORANGE,
        _                                              => if doc.is_encrypted { ui::RED_ALERT } else { ui::TEAL },
    };

    println!(
        "{} [ID: {}] CLASS: {} :: {}",
        color, doc.id, doc.clearance_level, doc.timestamp
    );

    if doc.is_encrypted {
        println!(" {}ENCRYPTED CONTENT - DECRYPTION REQUIRED{}", ui::RED_ALERT, ui::RESET);
        println!(" {}{}{}", ui::GREY_DIM, scramble_text(&doc.content, &mut rng), ui::RESET);
    } else {
        let content = corrupt_text(&doc.content, engine.turn_count, &mut rng);
        println!(" {}{}{}", color, content, ui::RESET);
    }

    if let Some(crate::state::CrisisUrgency::High) = &doc.crisis_urgency {
        println!(" {}[ !! UNRESOLVED — ESCALATES NEXT TURN !! ]{}", ui::ORANGE, ui::RESET);
    }

    println!("{}{}", ui::GREY_DIM, "─".repeat(60));
}
```

- [ ] **Step 5: Add `review` to the printed command list**

In the `AVAILABLE COMMANDS` println block, add:

```rust
println!("  [11] {}review -n [NAME]{}", ui::WHITE_BOLD, ui::RESET);
```

And in the `help` text, add: `"  review <NAME>       - View advisor advice history"`.

- [ ] **Step 6: Verify compilation**

```
cargo check
```

Expected: Clean.

- [ ] **Step 7: Commit**

```
git add src/main.rs
git commit -m "feat: act transition splashes, Basilisk awakening, updated HUD call, crisis doc rendering"
```

---

## Task 10: Ending evaluation and debrief display

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Replace the game-over block at the bottom of the loop**

Find the current game-over check at the bottom of the main loop:

```rust
if engine.state.is_terminal() {
    ui::clear_screen();
    println!("{}GAME OVER{}", ui::RED_ALERT, ui::RESET);
    break;
}
```

Replace with:

```rust
if engine.state.is_terminal() {
    let ending = engine.evaluate_ending()
        .unwrap_or(crate::state::Ending::NuclearWinter); // fallback
    let debrief = engine.build_debrief();

    ascii_art::play_ending(&ending, &mut rng);

    // Debrief screen
    println!("\n{}╔══════════════════ DECLASSIFIED DEBRIEF ══════════════════╗{}", ui::AMBER, ui::RESET);
    println!("{}  MOLE: {:<20} STATUS: {}{}", ui::AMBER, debrief.mole_name,
        if debrief.mole_caught { "NEUTRALIZED" } else { "ESCAPED" }, ui::RESET);
    println!("{}  FINAL DEFCON:    {:.2}{}",   ui::AMBER, debrief.final_tension,    ui::RESET);
    println!("{}  FINAL STABILITY: {:.2}{}",   ui::AMBER, debrief.final_stability,  ui::RESET);
    println!("{}  CORRUPTION:      {:.2}{}",   ui::AMBER, debrief.final_corruption, ui::RESET);
    println!("{}  PEAK TENSION AT: TURN {:02}{}",ui::AMBER, debrief.peak_tension_turn, ui::RESET);
    println!("{}╚═══════════════════════════════════════════════════════════╝{}", ui::AMBER, ui::RESET);

    println!("\n{}[PRESS ENTER TO EXIT]{}", ui::GREY_DIM, ui::RESET);
    let _ = input_mgr.read_line();
    break;
}
```

- [ ] **Step 2: Set `standdown_triggered` in the StandDown directive handler (game.rs)**

In `Directive::StandDown` arm of `resolve_directive`, add at the start:

```rust
self.standdown_triggered = true;
```

- [ ] **Step 3: Verify compilation**

```
cargo check
```

Expected: Clean.

- [ ] **Step 4: Full playthrough test**

```
cargo run
```

Play until any game-over condition. Verify:
- The correct ending ASCII art appears
- The debrief shows accurate mole name, caught status, and tension values
- The game exits cleanly after Enter

Test each ending path at least once:
- Let tension hit 1.0 → NUCLEAR WINTER
- Spam StandDown from low tension → COLD PEACE
- Let corruption climb → watch MACHINE WON trigger mid-turn silently

- [ ] **Step 5: Commit**

```
git add src/game.rs src/main.rs
git commit -m "feat: ending evaluation, ASCII art splashes, and declassified debrief screen"
```

---

## Self-Review

### Spec coverage check

| Spec requirement | Covered in task |
|-----------------|----------------|
| GameAct enum, state-triggered transitions | Tasks 1, 6, 9 |
| BasiliskStage from corruption | Tasks 1, 7 |
| Fake doc injection (AWARE+) | Task 7 |
| DEFCON scramble (AWARE+) | Task 3 |
| Command hijacking upgraded for AUTONOMOUS | Task 7 |
| HUD border glitch (AUTONOMOUS) | Task 3 |
| CrisisUrgency + crisis docs in document stream | Tasks 1, 5, 6 |
| Critical countdown timer + bell | Task 3 |
| Act-gated crisis frequency | Task 6 |
| advice_log on Advisor | Tasks 1, 8 |
| review command | Task 8 |
| Interception window (15s) replacing Red Phone | Task 8 |
| mole_silence_turns on failed interception | Task 8 |
| 6 endings with priority order | Tasks 6, 10 |
| Debrief screen | Task 10 |
| ASCII art: 3 acts, 6 endings, boot, awakening | Task 4 |
| play_boot_sequence replaces inline boot | Task 9 |
| Basilisk awakening scene at AWARE | Tasks 4, 9 |
| is_terminal() includes corruption >= 1.0 | Task 1 |
| InputManager::try_read_line | Task 2 |
| draw_hud accepts corruption | Task 3 |
| No new dependencies | All tasks |

### Type consistency check

- `CrisisUrgency::Critical(u32)` used consistently across state.rs, document.rs, game.rs, main.rs
- `GameAct` enum variants `Watchdog / Protocol / ZeroHour` consistent across all files
- `BasiliskStage::from_corruption` signature consistent: takes `f64`, returns `BasiliskStage`
- `run_countdown(u32, &str, &InputManager) -> Option<String>` — called in main.rs Task 8 Step 5 with matching args
- `draw_hud(u32, f64, u32, u32, f64, &mut SimpleRng)` — updated call site in Task 9 Step 3 matches new signature
- `generate_mole_transmission_doc(&str, &mut SimpleRng) -> Document` — called in Task 8 Step 5 matching
- `mole_transmission_target: String` — set as `String`, read as `&str` via `.clone()` — consistent
