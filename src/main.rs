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
    ui::clear_screen();
    ui::type_text(
        "INITIALIZING SECURE TERMINAL LINK...",
        30,
        ui::TEAL,
        0.0,
        &mut rng,
    );
    thread::sleep(Duration::from_millis(500));
    ui::type_text(
        "LOADING GEOPOLITICAL HEURISTICS...",
        20,
        ui::TEAL,
        0.05,
        &mut rng,
    );
    thread::sleep(Duration::from_millis(500));
    ui::type_text(
        "ESTABLISHING NEURAL HANDSHAKE...",
        20,
        ui::TEAL,
        0.1,
        &mut rng,
    );

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

        if !skip_generation {
            engine.start_turn();
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
            let color = if doc.is_encrypted {
                ui::RED_ALERT
            } else {
                ui::TEAL
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
                let content = corrupt_text(&doc.content, engine.turn_count, &mut rng);
                println!(" {}{}{}", ui::TEAL, content, ui::RESET);
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
  trace <NAME>        - Trace signal origin to advisor{}",
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

            let mut arg_id = None;
            if parts.len() > args_start_idx {
                arg_id = Some(parts[args_start_idx].to_string());
            } else if parts.len() > 1 {
                // Fallback for consult [name] where name is second part
                arg_id = Some(parts[parts.len() - 1].to_string());
            }

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
                ui::type_text(&line, 15, ui::TEAL, 0.02, &mut rng);
            }

            if turn_ended {
                println!("\n{}[PRESS ENTER TO PROCEED]{}", ui::TEAL, ui::RESET);
                let _ = input_mgr.read_line();
            }
        }

        if engine.state.is_terminal() {
            ui::clear_screen();
            println!("{}GAME OVER{}", ui::RED_ALERT, ui::RESET);
            break;
        }
    }
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
