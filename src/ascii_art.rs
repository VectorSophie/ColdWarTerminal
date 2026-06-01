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
        Ending::TheMachineWon   => (END_MACHINE_WON,    ui::RED_ALERT),
        Ending::NuclearWinter   => (END_NUCLEAR_WINTER,  ui::RED_ALERT),
        Ending::ThePurge        => (END_THE_PURGE,       ui::ORANGE),
        Ending::PyrrhicVictory  => (END_PYRRHIC_VICTORY, ui::AMBER),
        Ending::WatchdogSuccess => (END_WATCHDOG_SUCCESS, ui::TEAL),
        Ending::ColdPeace       => (END_COLD_PEACE,       ui::TEAL),
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
