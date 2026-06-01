mod ascii_art;
mod document;
mod game;
mod input;
mod rng;
mod state;
mod ui;

use game::{Directive, GameEngine};
use input::InputManager;
use rng::SimpleRng;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

// Legacy Color Mapping for Helper Functions (Removed unused constants)

fn main() {
    let mut engine = GameEngine::new();
    let mut rng = SimpleRng::new();
    let input_mgr = InputManager::new();
    let mut stdout = io::stdout();

    // Boot Sequence
    ascii_art::play_boot_sequence(&mut rng);

    let mut skip_generation = false;

    loop {
        // --- CRISIS CHECK: THE RED PHONE ---
        if engine.state.red_phone_active {
            handle_red_phone_crisis(&mut engine, &mut rng, &input_mgr);
            if engine.state.is_terminal() {
                break;
            }
            engine.state.red_phone_active = false;
        }

        // Mole transmission interception window
        if engine.mole_transmission_active {
            use crate::document::generate_mole_transmission_doc;
            let target = engine.mole_transmission_target.clone();
            let doc = generate_mole_transmission_doc(&target, &mut rng);

            ui::clear_screen();
            println!("{}", ui::RED_ALERT);
            println!("!!! {} !!!", doc.content);
            println!("{}", ui::RESET);

            print!("{}root@command:~$ {}", ui::TEAL, ui::RESET);
            stdout.flush().unwrap();
            let response = ui::run_countdown(15, "SIGNAL INTERCEPT", &input_mgr);

            let intercepted = if let Some(ref cmd) = response {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                let base = parts.first().map(|s| s.trim_start_matches('-').to_lowercase()).unwrap_or_default();
                // Find the last non-flag token as the target name
                let arg = parts.iter()
                    .filter(|s| !s.starts_with('-'))
                    .last()
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();
                let first_word = target.to_lowercase();
                let first_word = first_word.split_whitespace().next().unwrap_or("");
                (base == "trace" || base == "traceroute") && arg.contains(first_word)
            } else {
                false
            };

            engine.mole_transmission_active = false;
            engine.mole_transmission_target = String::new();

            if intercepted {
                println!("{}SIGNAL CUT. {} IN CUSTODY.{}", ui::TEAL, target.to_uppercase(), ui::RESET);
                engine.state.red_phone_active = true;
            } else {
                println!("{}TRANSMISSION COMPLETE. ENEMY HAS THE CODES.{}", ui::RED_ALERT, ui::RESET);
                if let Some(mole) = engine.state.advisors.iter_mut().find(|a| a.name == target) {
                    mole.suspicion = 60;
                }
                engine.state.global_tension += 0.2;
                engine.state.global_tension = engine.state.global_tension.clamp(0.0, 1.0);
                engine.mole_silence_turns = 2;
            }

            println!("\n{}[PRESS ENTER TO CONTINUE]{}", ui::TEAL, ui::RESET);
            let _ = input_mgr.read_line();
            skip_generation = true;
        }

        if !skip_generation {
            engine.start_turn();

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
        } else {
            skip_generation = false;
        }

        // --- RENDER DASHBOARD ---
        ui::clear_screen();
        ui::draw_hud(
            engine.turn_count,
            engine.state.global_tension,
            engine.intel_points,
            engine.max_intel_points,
            engine.state.system_corruption,
            &mut rng,
        );
        println!();

        // WORLD METRICS
        println!("{}SYSTEM STATUS:{}", ui::AMBER, ui::RESET);
        ui::draw_progress_bar(
            "STABILITY",
            engine.state.domestic_stability,
            40,
            ui::TEAL,
            &mut rng,
        );
        ui::draw_progress_bar(
            "PARANOIA",
            engine.state.foreign_paranoia,
            40,
            ui::ORANGE,
            &mut rng,
        );
        ui::draw_progress_bar(
            "SECRECY",
            engine.state.internal_secrecy,
            40,
            ui::TEAL,
            &mut rng,
        );

        if engine.state.system_corruption > 0.0 {
            ui::draw_progress_bar(
                "SYS.CORRUPTION",
                engine.state.system_corruption,
                40,
                ui::RED_ALERT,
                &mut rng,
            );
        }

        println!();
        println!("{}ADVISOR LOYALTY:{}", ui::AMBER, ui::RESET);
        for advisor in &engine.state.advisors {
            let color = if advisor.suspicion > 70 {
                ui::RED_ALERT
            } else {
                ui::TEAL
            };
            ui::draw_progress_bar(
                &advisor.name,
                advisor.suspicion as f64 / 100.0,
                40,
                color,
                &mut rng,
            );
        }

        println!();
        println!("{}INCOMING TRANSMISSIONS:{}", ui::WHITE_BOLD, ui::RESET);
        println!("{}{}", ui::GREY_DIM, "─".repeat(60));

        // Interruption Check
        if engine.interruption_active && rng.random_bool(0.3) {
            trigger_interruption(&mut rng, &input_mgr);
        }

        // Display Documents
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
                println!(
                    " {}ENCRYPTED CONTENT - DECRYPTION REQUIRED{}",
                    ui::RED_ALERT,
                    ui::RESET
                );
                println!(
                    " {}{}{}",
                    ui::GREY_DIM,
                    scramble_text(&doc.content, &mut rng),
                    ui::RESET
                );
            } else {
                // Corruption only kicks in at Interfering stage (50%+), not from turn count
                let glitch = if matches!(
                    crate::state::BasiliskStage::from_corruption(engine.state.system_corruption),
                    crate::state::BasiliskStage::Interfering | crate::state::BasiliskStage::Autonomous
                ) {
                    corrupt_text(&doc.content, engine.turn_count, &mut rng)
                } else {
                    doc.content.clone()
                };
                ui::type_text(&format!(" {}", glitch), 5, color, 0.0, &mut rng);
            }

            if let Some(crate::state::CrisisUrgency::High) = &doc.crisis_urgency {
                println!(" {}[ !! UNRESOLVED — ESCALATES NEXT TURN !! ]{}", ui::ORANGE, ui::RESET);
            }

            println!("{}{}", ui::GREY_DIM, "─".repeat(60));
        }
        println!("{}", ui::RESET);

        // Input Phase
        println!(
            "\n{}AVAILABLE COMMANDS (Type 'help' for syntax):{}",
            ui::AMBER,
            ui::RESET
        );
        println!("  [1] {}sudo --escalate{}", ui::WHITE_BOLD, ui::RESET);
        println!("  [2] {}sudo --investigate{}", ui::WHITE_BOLD, ui::RESET);
        println!("  [3] {}sudo --contain{}", ui::WHITE_BOLD, ui::RESET);
        println!("  [4] {}sudo --leak{}", ui::WHITE_BOLD, ui::RESET);
        println!("  [5] {}sudo --stand-down{}", ui::WHITE_BOLD, ui::RESET);
        println!("  [6] {}decrypt -t [ID]{}", ui::WHITE_BOLD, ui::RESET);
        println!("  [7] {}analyze -t [ID]{}", ui::WHITE_BOLD, ui::RESET);
        println!("  [8] {}traceroute -t [NAME]{}", ui::WHITE_BOLD, ui::RESET);
        println!("  [9] {}consult -n [NAME]{}", ui::WHITE_BOLD, ui::RESET);
        println!(
            "  [10] {}interrogate -n [NAME]{}",
            ui::WHITE_BOLD,
            ui::RESET
        );
        println!("  [11] {}review -n [NAME]{}", ui::WHITE_BOLD, ui::RESET);

        let directive;
        loop {
            print!("{}root@command:~$ {}", ui::TEAL, ui::RESET);
            stdout.flush().unwrap();

            let input = input_mgr.read_line();
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            if input == "clear" || input == "cls" {
                skip_generation = true;
                directive = None;
                break;
            }
            if input == "help" {
                println!(
                    "{}Available Commands:
  escalate      - Increase military readiness (High Risk)
  investigate   - Root out internal threats
  contain       - Attempt diplomatic de-escalation
  leak          - Release information to public
  stand-down    - Withdraw military forces (Surrender)
  decrypt <ID>  - Decrypt intelligence document
  analyze <ID>  - Verify document reliability
  consult <NAME>      - Ask advisor for counsel
  interrogate <NAME>  - Aggressively question advisor
  trace <NAME>        - Trace signal origin to advisor
  review <NAME>       - View advisor advice history{}",
                    ui::GREY_DIM,
                    ui::RESET
                );
                continue;
            }

            let parts: Vec<&str> = input.split_whitespace().collect();
            let cmd_base = parts.get(0).unwrap_or(&"").to_lowercase();
            let (mut command_str, args_start_idx) = if cmd_base == "sudo" || cmd_base == "execute" {
                (parts.get(1).unwrap_or(&"").to_lowercase(), 2)
            } else {
                (cmd_base.clone(), 1)
            };

            // Handle flags (strip leading dashes)
            let cleaned_cmd = command_str.trim_start_matches("-").to_string();
            command_str = cleaned_cmd;

            // Pick the last non-flag token after the command (handles -n, -t prefixes)
            let arg_id = parts[args_start_idx..]
                .iter()
                .filter(|s| !s.starts_with('-'))
                .last()
                .map(|s| s.to_string());

            let d = match command_str.as_str() {
                "1" | "escalate" | "esc" => Some(Directive::Escalate),
                "2" | "investigate" | "inv" => Some(Directive::Investigate),
                "3" | "contain" | "con" => Some(Directive::Contain),
                "4" | "leak" => Some(Directive::Leak),
                "5" | "stand-down" | "standdown" | "sd" => Some(Directive::StandDown),
                "6" | "decrypt" | "dec" => {
                    if let Some(id) = arg_id {
                        Some(Directive::Decrypt(id))
                    } else {
                        println!("usage: decrypt -t <id>");
                        continue;
                    }
                }
                "7" | "analyze" | "ana" => {
                    if let Some(id) = arg_id {
                        Some(Directive::Analyze(id))
                    } else {
                        println!("usage: analyze -t <id>");
                        continue;
                    }
                }
                "8" | "trace" | "traceroute" => {
                    if let Some(id) = arg_id {
                        Some(Directive::Trace(id))
                    } else {
                        println!("usage: traceroute -t <advisor_name>");
                        continue;
                    }
                }
                "9" | "consult" => {
                    if let Some(id) = arg_id {
                        Some(Directive::Consult(id))
                    } else {
                        println!("usage: consult -n <advisor_name>");
                        continue;
                    }
                }
                "10" | "interrogate" | "int" => {
                    if let Some(id) = arg_id {
                        Some(Directive::Interrogate(id))
                    } else {
                        println!("usage: interrogate -n <advisor_name>");
                        continue;
                    }
                }
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
                "quit" | "exit" => std::process::exit(0),
                _ => {
                    println!(
                        "Unknown command: '{}'. Type 'help' for options.",
                        command_str
                    );
                    continue;
                }
            };

            if let Some(dir) = d {
                directive = Some(dir);
                break;
            }
        }

        if let Some(dir) = directive {
            let (feedback, turn_ended) = engine.resolve_directive(dir);
            skip_generation = !turn_ended;

            println!("\n{}EXECUTING DIRECTIVE...{}", ui::AMBER, ui::RESET);
            for line in feedback {
                ui::type_text(&line, 30, ui::TEAL, 0.02, &mut rng);
            }

            if turn_ended {
                println!("\n{}[PRESS ENTER TO PROCEED]{}", ui::TEAL, ui::RESET);
                let _ = input_mgr.read_line();
            }
        }

        if engine.state.is_terminal() {
            let ending = engine.evaluate_ending()
                .unwrap_or(crate::state::Ending::NuclearWinter);
            let debrief = engine.build_debrief();
            display_ending_and_debrief(&engine, &ending, &debrief, &mut rng, &input_mgr);
            break;
        }

        // Also check non-fatal endings (e.g. ColdPeace from StandDown)
        if engine.standdown_triggered {
            if let Some(ending) = engine.evaluate_ending() {
                let debrief = engine.build_debrief();
                display_ending_and_debrief(&engine, &ending, &debrief, &mut rng, &input_mgr);
                break;
            }
        }
    }
}

fn display_ending_and_debrief(
    _engine: &GameEngine,
    ending: &crate::state::Ending,
    debrief: &crate::state::DebriefData,
    rng: &mut SimpleRng,
    input_mgr: &InputManager,
) {
    ascii_art::play_ending(ending, rng);

    println!("\n{}╔══════════════════ DECLASSIFIED DEBRIEF ══════════════════╗{}", ui::AMBER, ui::RESET);
    println!("{}  MOLE:             {:<20} STATUS: {}{}", ui::AMBER, debrief.mole_name,
        if debrief.mole_caught { "NEUTRALIZED" } else { "ESCAPED" }, ui::RESET);
    println!("{}  FINAL DEFCON:    {:.2}{}", ui::AMBER, debrief.final_tension, ui::RESET);
    println!("{}  FINAL STABILITY: {:.2}{}", ui::AMBER, debrief.final_stability, ui::RESET);
    println!("{}  CORRUPTION:      {:.2}{}", ui::AMBER, debrief.final_corruption, ui::RESET);
    println!("{}  PEAK TENSION AT: TURN {:02}{}", ui::AMBER, debrief.peak_tension_turn, ui::RESET);
    println!("{}╚═══════════════════════════════════════════════════════════╝{}", ui::AMBER, ui::RESET);

    println!("\n{}[PRESS ENTER TO EXIT]{}", ui::GREY_DIM, ui::RESET);
    let _ = input_mgr.read_line();
}

fn handle_red_phone_crisis(
    engine: &mut GameEngine,
    _rng: &mut SimpleRng,
    input_mgr: &InputManager,
) {
    let is_mole_reveal = engine.state.advisors.iter().any(|a| a.suspicion >= 100);

    ui::clear_screen();
    println!("{}INCOMING PRIORITY ONE ALERT", ui::RED_ALERT);
    thread::sleep(Duration::from_millis(500));
    println!("\n{}CONNECTION ESTABLISHED.{}", ui::RED_ALERT, ui::RESET);

    if is_mole_reveal {
        println!(
            "{}VOICE: So... you figured it out. Smart.{}",
            ui::AMBER,
            ui::RESET
        );
        thread::sleep(Duration::from_millis(2000));
        println!("{}VOICE: I am doing this for the greater good. The war is inevitable. I just wanted to finish it quickly.{}", ui::AMBER, ui::RESET);
        println!("\nDECISION POINT:");
        println!("1. EXECUTE (Silence the traitor. Immediate stability boost, high paranoia.)");
        println!("2. TURN (Force them to double-agent. High risk, high intel reward.)");

        print!("\n{}YOUR ORDER >> {}", ui::RED_ALERT, ui::RESET);
        io::stdout().flush().unwrap();

        input_mgr.flush();
        let input = input_mgr.read_line();
        let input = input.trim();

        match input {
            "1" | "execute" => {
                println!(
                    "\n{}COMMAND: SECURITY TEAM DISPATCHED. TARGET NEUTRALIZED.{}",
                    ui::TEAL,
                    ui::RESET
                );
                engine.state.domestic_stability += 0.3;
                engine.state.foreign_paranoia += 0.2;
            }
            _ => {
                println!(
                    "\n{}COMMAND: ASSET FLIPPED. THEY ARE FEEDING DISINFORMATION TO THE ENEMY.{}",
                    ui::TEAL,
                    ui::RESET
                );
                engine.state.global_tension -= 0.3;
                engine.state.internal_secrecy -= 0.1;
                engine.state.accidental_escalation_risk += 0.1;
            }
        }
        if let Some(mole_mut) = engine
            .state
            .advisors
            .iter_mut()
            .find(|a| a.suspicion >= 100)
        {
            mole_mut.suspicion = 0;
            mole_mut.is_mole = false;
        }
        engine.mole_neutralized = true;
    } else {
        println!(
            "{}VOICE: PREMIER CHERNOV HERE. WE SEE YOUR BOMBERS. EXPLAIN YOURSELF OR WE LAUNCH.{}",
            ui::AMBER,
            ui::RESET
        );
        println!("(You have 10 seconds to respond correctly)");
        println!("\nDECISION POINT:");
        println!("1. DENY (Claim it's a training exercise)");
        println!("2. ADMIT (Tell the truth, ask for de-escalation)");
        println!("3. THREATEN (Tell them to back down or else)");

        print!("\n{}YOUR RESPONSE >> {}", ui::RED_ALERT, ui::RESET);
        io::stdout().flush().unwrap();

        input_mgr.flush();
        let input = input_mgr.read_line();
        let input = input.trim();

        match input {
            "1" | "deny" => {
                if engine.state.foreign_paranoia > 0.7 {
                    println!(
                        "\n{}CHERNOV: LIAR! WE ARE LAUNCHING!{}",
                        ui::RED_ALERT,
                        ui::RESET
                    );
                    engine.state.global_tension = 1.0;
                } else {
                    println!(
                        "\n{}CHERNOV: ...Fine. Turn them around. Now.{}",
                        ui::AMBER,
                        ui::RESET
                    );
                    engine.state.global_tension -= 0.2;
                }
            }
            "2" | "admit" => {
                println!("\n{}CHERNOV: A bold admission. We will stand down, but there will be consequences.{}", ui::AMBER, ui::RESET);
                engine.state.global_tension -= 0.5;
                engine.state.domestic_stability -= 0.3;
            }
            "3" | "threaten" => {
                println!("\n{}CHERNOV: THEN LET IT END!{}", ui::RED_ALERT, ui::RESET);
                engine.state.global_tension = 1.0;
            }
            _ => {
                println!(
                    "\n{}CHERNOV: YOUR SILENCE IS DAMNING. LAUNCHING!{}",
                    ui::RED_ALERT,
                    ui::RESET
                );
                engine.state.global_tension = 1.0;
            }
        }
    }

    thread::sleep(Duration::from_millis(3000));
    println!("{}CALL TERMINATED.{}", ui::RED_ALERT, ui::RESET);
    thread::sleep(Duration::from_millis(2000));
}

fn corrupt_text(text: &str, turn: u32, rng: &mut SimpleRng) -> String {
    if turn < 8 {
        return text.to_string();
    }
    let probability = if turn < 12 {
        0.05
    } else if turn < 16 {
        0.15
    } else {
        0.30
    };
    text.chars()
        .map(|c| {
            if c.is_whitespace() {
                c
            } else if rng.random_bool(probability) {
                match rng.range(0, 5) {
                    0 => '#',
                    1 => '_',
                    2 => '?',
                    3 => '%',
                    _ => ' ',
                }
            } else {
                c
            }
        })
        .collect()
}

fn random_char(rng: &mut SimpleRng) -> char {
    let chars = b"0123456789ABCDEFXZ@#&";
    let idx = rng.range(0, chars.len() as u64) as usize;
    chars[idx] as char
}

fn scramble_text(text: &str, rng: &mut SimpleRng) -> String {
    let mut s = String::new();
    for c in text.chars() {
        if c.is_whitespace() {
            s.push(' ');
        } else {
            s.push(random_char(rng));
        }
    }
    s
}

fn trigger_interruption(_rng: &mut SimpleRng, _input_mgr: &InputManager) {
    print!("\x07");
    println!(
        "\n{}!!! SIGNAL INTERRUPT DETECTED !!!{}",
        ui::RED_ALERT,
        ui::RESET
    );
    thread::sleep(Duration::from_millis(500));
    // ASCII Art omitted for brevity in rewrite, just a message
    println!(
        "{}INTRUDER MESSAGE: THEY ARE WATCHING.{}",
        ui::RED_ALERT,
        ui::RESET
    );
    thread::sleep(Duration::from_millis(1000));
}
