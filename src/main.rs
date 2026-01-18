mod state;
mod document;
mod game;
mod rng;

use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use game::{GameEngine, Directive};
use rng::SimpleRng;

// ANSI Colors
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";

fn main() {
    let mut engine = GameEngine::new();
    let mut rng = SimpleRng::new(); 
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("{}========================================", GREEN);
    println!("      C O L D   W A R   T E R M I N A L");
    println!("========================================{}", RESET);
    println!("Authenticating user... CLEARED: LEVEL 5");
    println!("Loading world state...");
    println!("");

    let mut skip_generation = false;

    loop {
        // --- CRISIS CHECK: THE RED PHONE ---
        if engine.state.red_phone_active {
            handle_red_phone_crisis(&mut engine, &mut rng);
            if engine.state.is_terminal() { break; } // Check if game ended inside crisis
            // If survived, reset flag and continue
            engine.state.red_phone_active = false;
        }

        // 1. Start Turn & Generate Docs (unless we just did a minor action)
        if !skip_generation {
            engine.start_turn();
        } else {
            skip_generation = false; // Reset
        }

        // 2. Display Status
        println!("\n{}--- TURN {} REPORT ---", CYAN, engine.turn_count);
        println!("DEFCON ESTIMATE: {}", defcon_level(engine.state.global_tension));
        println!("DOMESTIC MOOD:   {}{}", stability_desc(engine.state.domestic_stability), RESET);
        
        // Display Intel Points
        print!("INTEL ASSETS:    [");
        for _ in 0..engine.intel_points { print!("{}#{}", YELLOW, RESET); }
        for _ in 0..(engine.max_intel_points - engine.intel_points) { print!("."); }
        println!("]");

        // Display Advisors
        println!("\n{}ADVISOR LOYALTY STATUS:{}", MAGENTA, RESET);
        for advisor in &engine.state.advisors {
            let suspicion_bar = (advisor.suspicion as f64 / 10.0).round() as usize;
            let color = if advisor.suspicion > 70 { RED } else { GREEN };
            print!("{:<15} [", advisor.name);
            print!("{}", color);
            for _ in 0..suspicion_bar { print!("!"); }
            print!("{}", RESET);
            for _ in 0..(10 - suspicion_bar) { print!("."); }
            println!("] {} SUSPICION", advisor.suspicion);
        }
        
        println!("{}----------------------{}", CYAN, RESET);

        // 3. Display Documents
        println!("\n{}INCOMING CABLES:{}", BOLD, RESET);
        for doc in &engine.pending_documents {
            // Screen Shake Effect at High Tension
            let padding = if engine.state.global_tension > 0.7 {
                 let shake = rng.range(0, 4);
                 (0..shake).map(|_| " ").collect::<String>()
            } else {
                "".to_string()
            };

            // Interruption Logic (Check engine flag first, but RNG for visual effect)
            if engine.interruption_active && rng.random_bool(0.3) {
                trigger_interruption(&mut rng);
            }

            println!("\n{}{}[ID: {} | CLASS: {} | TIME: {}]{}", 
                padding, CYAN, doc.id, doc.clearance_level, doc.timestamp, RESET);
            
            print!("{}> ", padding);
            stdout.flush().unwrap();

            if doc.is_encrypted {
                print!("{}", RED);
                print_slowly(&scramble_text(&doc.content, &mut rng), 5); // Faster scramble print
                print!("{}", RESET);
                println!("{}   [ENCRYPTED CONTENT - DECRYPTION REQUIRED]{}", RED, RESET);
            } else {
                print!("{}", GREEN);
                print_slowly(&doc.content, 35);
                print!("{}", RESET);
            }
        }

        // 4. Input Phase
        println!("\n{}AVAILABLE COMMANDS (Type 'help' for syntax):{}", YELLOW, RESET);
        println!("  [1] {}execute --escalate{}", BOLD, RESET);
        println!("  [2] {}execute --investigate{}", BOLD, RESET);
        println!("  [3] {}execute --contain{}", BOLD, RESET);
        println!("  [4] {}execute --leak{}", BOLD, RESET);
        println!("  [5] {}execute --stand-down{}", BOLD, RESET);
        println!("  [6] {}decrypt -t [ID]{}", BOLD, RESET);
        println!("  [7] {}analyze -t [ID]{}", BOLD, RESET);
        println!("  [8] {}traceroute{}", BOLD, RESET);
        
        print!("\n{}root@command:~$ {}", GREEN, RESET);
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).expect("Failed to read input");
        let input = input.trim();
        
        // Handle common Linux commands for immersion
        if input == "clear" || input == "cls" {
            print!("{}", RESET);
            for _ in 0..50 { println!(); }
            continue;
        }
        if input == "ls" || input == "ll" {
            println!("{}drwx------ 2 root root 4096 .secrets{}", CYAN, RESET);
            println!("{}drwx------ 2 root root 4096 .basilisk{}", CYAN, RESET);
            println!("{}drwxr-xr-x 2 root root 4096 cables{}", CYAN, RESET);
            continue;
        }
        if input == "whoami" {
            println!("{}root (Security Clearance Level 5){}", MAGENTA, RESET);
            continue;
        }
        if input == "help" {
             println!("Usage: command [options] [target]");
             println!("Aliases accepted: esc, inv, con, leak, sd, dec, ana, trace");
             continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd_base = parts.get(0).unwrap_or(&"").to_lowercase();
        
        // Handle "sudo" prefix
        let (command_str, args_start_idx) = if cmd_base == "sudo" || cmd_base == "execute" {
            (parts.get(1).unwrap_or(&"").to_lowercase(), 2)
        } else {
            (cmd_base, 1)
        };

        // Extract ID argument (skipping flags like -t, --target)
        let mut arg_id = None;
        for part in parts.iter().skip(args_start_idx - 1) { // Look through all parts
            if part.starts_with("DOC-") || part.starts_with("SIGNAL-") {
                arg_id = Some(part.to_string());
                break;
            }
        }
        // Fallback: take the last arg if it looks like an ID or just the second word
        if arg_id.is_none() && parts.len() > 1 {
             let last = parts.last().unwrap();
             if !last.starts_with("-") {
                 arg_id = Some(last.to_string());
             }
        }

        let directive = match command_str.as_str() {
            "1" | "escalate" | "esc" | "--escalate" => Directive::Escalate,
            "2" | "investigate" | "inv" | "--investigate" | "audit" => Directive::Investigate,
            "3" | "contain" | "con" | "--contain" => Directive::Contain,
            "4" | "leak" | "--leak" | "pub" => Directive::Leak,
            "5" | "stand-down" | "standdown" | "sd" | "--stand-down" | "abort" => Directive::StandDown,
            "6" | "decrypt" | "dec" | "crack" | "cat" => {
                if let Some(id) = arg_id {
                    Directive::Decrypt(id)
                } else {
                    println!("{}ERROR: MISSING TARGET. USAGE: decrypt -t DOC-XXXX{}", RED, RESET);
                    continue; 
                }
            },
            "7" | "analyze" | "ana" | "stat" | "check" => {
                if let Some(id) = arg_id {
                    Directive::Analyze(id)
                } else {
                     println!("{}ERROR: MISSING TARGET. USAGE: analyze -t DOC-XXXX{}", RED, RESET);
                     continue;
                }
            },
            "8" | "trace" | "traceroute" | "netstat" | "tr" => Directive::Trace,
            "quit" | "exit" => break,
            _ => {
                println!("{}BASH: COMMAND NOT FOUND: {}. DEFAULTING TO 'CONTAIN'.{}", RED, command_str, RESET);
                Directive::Contain
            }
        };

        // 5. Resolve
        let (feedback, turn_ended) = engine.resolve_directive(directive);
        
        skip_generation = !turn_ended;

        println!("\n{}EXECUTING DIRECTIVE...{}", YELLOW, RESET);
        for line in feedback {
            if line.starts_with("CONTENT: ") {
                let content = &line["CONTENT: ".len()..];
                print!(" :: ");
                stdout.flush().unwrap();
                animate_decryption(content, &mut rng);
            } else {
                print!(" :: ");
                stdout.flush().unwrap();
                print_slowly(&line, 35);
            }
        }

        // 6. Check End State
        if engine.state.is_terminal() {
            println!("\n{}========================================", RED);
            if engine.state.global_tension >= 1.0 {
                println!("GAME OVER: NUCLEAR LAUNCH DETECTED.");
                println!("The world ends in fire.");
            } else if engine.state.domestic_stability <= 0.0 {
                println!("GAME OVER: GOVERNMENT COLLAPSE.");
                println!("You have been removed from office by a military coup.");
            }
            println!("Turns Survived: {}", engine.turn_count);
            println!("========================================{}", RESET);
            break;
        }

        // 7. Divergent Ending Check (Basilisk)
        if engine.state.secret_weapon_progress >= 1.0 {
            transition_phase(&engine);
            println!("\n{}========================================", RED);
            println!("GAME OVER: REALITY FAILURE.");
            println!("Project Basilisk has achieved consciousness.");
            println!("It has calculated that the only path to peace is the removal of humanity.");
            println!("========================================{}", RESET);
            break;
        }

        if engine.turn_count >= 20 {
             println!("\n[SIMULATION END: MAX TURNS REACHED]");
             break;
        }

        // 8. End of Day Transition
        if turn_ended {
            transition_phase(&engine);
        }
    }
}

fn handle_red_phone_crisis(engine: &mut GameEngine, _rng: &mut SimpleRng) {
    // Determine the nature of the crisis
    let is_mole_reveal = engine.state.advisors.iter().any(|a| a.suspicion >= 100);
    
    // Clear screen hard
    print!("{}", RESET);
    for _ in 0..100 { println!(); }
    
    // Ringing effect
    println!("{}INCOMING PRIORITY ONE ALERT", RED);
    for _ in 0..5 {
        print!("R I N G . . . ");
        io::stdout().flush().unwrap();
        print!("\x07"); // Beep
        thread::sleep(Duration::from_millis(800));
        print!("\r                    \r"); // Clear line
        thread::sleep(Duration::from_millis(200));
    }

    println!("\n{}CONNECTION ESTABLISHED.{}", RED, RESET);
    
    if is_mole_reveal {
        // MOLE CONFRONTATION
        let _mole = engine.state.advisors.iter().find(|a| a.suspicion >= 100).unwrap();
        println!("{}VOICE: So... you figured it out. Smart.{}", YELLOW, RESET);
        thread::sleep(Duration::from_millis(2000));
        println!("{}VOICE: I am doing this for the greater good. The war is inevitable. I just wanted to finish it quickly.{}", YELLOW, RESET);
        println!("\nDECISION POINT:");
        println!("1. EXECUTE (Silence the traitor. Immediate stability boost, high paranoia.)");
        println!("2. TURN (Force them to double-agent. High risk, high intel reward.)");
        
        print!("\n{}YOUR ORDER >> {}", RED, RESET);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read");
        let input = input.trim();
        
        match input {
            "1" | "execute" => {
                println!("\n{}COMMAND: SECURITY TEAM DISPATCHED. TARGET NEUTRALIZED.{}", GREEN, RESET);
                engine.state.domestic_stability += 0.3;
                engine.state.foreign_paranoia += 0.2;
                // Remove the mole/advisor logic effectively handled by just continuing
                // In a full game, we'd remove them from the vec, but for prototype we reset suspicion
                // so the event doesn't loop.
            },
            _ => { // Turn or invalid
                println!("\n{}COMMAND: ASSET FLIPPED. THEY ARE FEEDING DISINFORMATION TO THE ENEMY.{}", GREEN, RESET);
                engine.state.global_tension -= 0.3;
                engine.state.internal_secrecy -= 0.1;
                engine.state.accidental_escalation_risk += 0.1;
            }
        }
        // Clear suspicion so event doesn't re-trigger immediately
        if let Some(mole_mut) = engine.state.advisors.iter_mut().find(|a| a.suspicion >= 100) {
            mole_mut.suspicion = 0;
            mole_mut.is_mole = false; // Resolved
        }

    } else {
        // NUCLEAR STANDOFF
        println!("{}VOICE: PREMIER CHERNOV HERE. WE SEE YOUR BOMBERS. EXPLAIN YOURSELF OR WE LAUNCH.{}", YELLOW, RESET);
        println!("(You have 10 seconds to respond correctly)");
        
        println!("\nDECISION POINT:");
        println!("1. DENY (Claim it's a training exercise)");
        println!("2. ADMIT (Tell the truth, ask for de-escalation)");
        println!("3. THREATEN (Tell them to back down or else)");
        
        print!("\n{}YOUR RESPONSE >> {}", RED, RESET);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read");
        let input = input.trim();
        
        match input {
            "1" | "deny" => {
                if engine.state.foreign_paranoia > 0.7 {
                    println!("\n{}CHERNOV: LIAR! WE ARE LAUNCHING!{}", RED, RESET);
                    engine.state.global_tension = 1.0; // Game Over
                } else {
                    println!("\n{}CHERNOV: ...Fine. Turn them around. Now.{}", YELLOW, RESET);
                    engine.state.global_tension -= 0.2;
                }
            },
            "2" | "admit" => {
                println!("\n{}CHERNOV: A bold admission. We will stand down, but there will be consequences.{}", YELLOW, RESET);
                engine.state.global_tension -= 0.5;
                engine.state.domestic_stability -= 0.3; // You look weak
            },
            "3" | "threaten" => {
                println!("\n{}CHERNOV: THEN LET IT END!{}", RED, RESET);
                engine.state.global_tension = 1.0; // Game Over
            },
            _ => {
                println!("\n{}CHERNOV: YOUR SILENCE IS DAMNING. LAUNCHING!{}", RED, RESET);
                engine.state.global_tension = 1.0;
            }
        }
    }
    
    thread::sleep(Duration::from_millis(3000));
    println!("{}CALL TERMINATED.{}", RED, RESET);
    thread::sleep(Duration::from_millis(2000));
}

fn transition_phase(engine: &GameEngine) {
    // "Pitch Black" - Clear screen by scrolling
    print!("{}", RESET);
    for _ in 0..50 { println!(); }
    
    // Silence/Darkness for 1.5 seconds
    thread::sleep(Duration::from_millis(1500));

    // Audio Cue (Day End)
    print!("\x07"); 
    io::stdout().flush().unwrap();

    // Dramatic Summary
    println!("{}========================================", CYAN);
    println!("      DAY {} SEQUENCE COMPLETED", engine.turn_count);
    println!("========================================{}", RESET);
    
    thread::sleep(Duration::from_millis(800));

    // Tension Bar Animation
    print!("\nGLOBAL TENSION: [");
    io::stdout().flush().unwrap();
    
    let bar_width: usize = 25;
    let filled = (engine.state.global_tension * bar_width as f64).round() as usize;
    let empty = bar_width.saturating_sub(filled);
    
    let color = if engine.state.global_tension > 0.75 { RED } 
               else if engine.state.global_tension > 0.4 { YELLOW } 
               else { GREEN };
    
    // Animate the bar filling up
    print!("{}", color);
    for _ in 0..filled { 
        print!("="); 
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(50));
    }
    print!("{}", RESET);
    for _ in 0..empty { print!(" "); }
    
    println!("] {:.0}%", engine.state.global_tension * 100.0);

    // Contextual Status Message
    if engine.state.global_tension > 0.8 {
        println!("{}STATUS: CRITICAL THRESHOLD IMMINENT. DEFCON 1 PREPARED.{}", RED, RESET);
    } else if engine.state.global_tension > 0.6 {
        println!("{}STATUS: ESCALATION DETECTED. FORCES ON HIGH ALERT.{}", YELLOW, RESET);
    } else if engine.state.global_tension < 0.3 {
        println!("{}STATUS: GEOPOLITICAL CLIMATE STABLE.{}", GREEN, RESET);
    }

    // Pause to let player read
    thread::sleep(Duration::from_millis(2500));

    // Clear again for fresh day start
    for _ in 0..50 { println!(); }
}

fn print_slowly(text: &str, delay_ms: u64) {
    for c in text.chars() {
        print!("{}", c);
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(delay_ms));
    }
    println!();
}

fn animate_decryption(target_text: &str, rng: &mut SimpleRng) {
    let target_chars: Vec<char> = target_text.chars().collect();
    let len = target_chars.len();
    
    // Initial scrambled state
    let mut current_display: Vec<char> = scramble_text(target_text, rng).chars().collect();
    
    if current_display.len() != len {
        current_display = vec!['#'; len];
    }

    for i in 0..len {
        if target_chars[i].is_whitespace() {
            current_display[i] = ' ';
            continue;
        }

        for _ in 0..4 { 
            current_display[i] = random_char(rng);
            let noise_idx = rng.range(i as u64, len as u64) as usize;
            if !target_chars[noise_idx].is_whitespace() {
                current_display[noise_idx] = random_char(rng);
            }

            let solved: String = current_display[0..i].iter().collect();
            let spinning = current_display[i];
            let unsolved: String = current_display[i+1..].iter().collect();

            print!("\r{}{}{}{}{}{}{}", 
                GREEN, solved, 
                YELLOW, spinning, 
                RED, unsolved,
                RESET
            );
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(15));
        }

        current_display[i] = target_chars[i];
        
        let solved: String = current_display[0..=i].iter().collect();
        let unsolved: String = current_display[i+1..].iter().collect();
        print!("\r{}{}{}{}{}", GREEN, solved, RED, unsolved, RESET);
        io::stdout().flush().unwrap();
    }
    println!();
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

fn trigger_interruption(rng: &mut SimpleRng) {
    print!("\x07"); // SYSTEM BELL
    println!("\n{}!!! SIGNAL INTERRUPT DETECTED !!!{}", RED, RESET);
    thread::sleep(Duration::from_millis(500));
    
    let ascii_art = match rng.range(0, 3) {
        // DOS Rebel font from Text to ASCII generator
        0 => r#"
 ███████████                     ███  ████   ███          █████     
░░███░░░░░███                   ░░░  ░░███  ░░░          ░░███      
 ░███    ░███  ██████    █████  ████  ░███  ████   █████  ░███ █████
 ░██████████  ░░░░░███  ███░░  ░░███  ░███ ░░███  ███░░   ░███░░███ 
 ░███░░░░░███  ███████ ░░█████  ░███  ░███  ░███ ░░█████  ░██████░  
 ░███    ░███ ███░░███  ░░░░███ ░███  ░███  ░███  ░░░░███ ░███░░███ 
 ███████████ ░░████████ ██████  █████ █████ █████ ██████  ████ █████
░░░░░░░░░░░   ░░░░░░░░ ░░░░░░  ░░░░░ ░░░░░ ░░░░░ ░░░░░░  ░░░░ ░░░░░ "#,
        // from https://emojicombos.com/eye-ascii-art
        1 => r#"
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠀⠀⠀⠀⠀⢀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⡇⠀⠀⢰⡆⢘⣆⠀⠀⡆⠀⢸⠀⢀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠀⣆⣧⡤⠾⢷⡚⠛⢻⣏⢹⡏⠉⣹⠟⡟⣾⠳⣼⢦⣀⣰⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠰⣄⡬⢷⣝⢯⣷⢤⣘⣿⣦⣼⣿⣾⣷⣼⣽⣽⣿⣯⡾⢃⣠⣞⠟⠓⢦⣀⠆⠀⠀⡀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠲⣄⣤⣞⡉⠛⢶⣾⡷⠟⣿⣿⣿⣿⣿⣿⣿⡿⣿⣿⣿⡿⢿⡛⠻⠿⣥⣤⣶⠞⠉⢓⣤⡴⢁⠄⠀⠀⠀⠀⠀
⠀⠀⠀⣄⣠⠞⠉⢛⣻⡿⠛⠁⠀⣸⠯⠈⠀⠁⣴⣿⣿⣿⡶⠤⠽⣇⠈⣿⠀⠀⠈⠙⠻⢶⣾⣻⣭⠿⢫⣀⣴⡶⠃⠀⠀
⠀⢤⣀⣜⣉⣩⣽⠿⠋⠀⠀⠀⠀⣿⠈⠀⠀⢸⣿⣿⣿⣿⣀⠀⠀⠸⠇⢸⡇⠀⠀⠀⠀⠀⠘⠛⢶⣶⣾⣻⡯⠄⠀⣠⠄
⠀⠤⠬⢭⣿⣿⠋⠀⠀⠀⠀⠀⠀⢻⡀⠀⠀⠀⢿⣿⣿⣿⡿⠋⠁⠀⠀⣼⠁⠀⠀⠀⠀⠀⢀⣴⣫⣏⣙⠛⠒⠚⠋⠁⠀
⡔⢀⡵⠋⢧⢹⡀⠀⠀⠀⠀⠀⠀⠈⢷⡀⠀⠀⠀⠈⠉⠉⠀⠀⠀⠀⣰⠏⠀⠀⠀⠀⠀⣠⣾⣿⡛⠛⠛⠓⠦⠀⠀⠀⠀
⣇⠘⠳⠦⠼⠧⠷⣄⣀⠀⠀⠀⠀⠀⠀⠳⢤⣀⠀⠀⠀⠀⠀⢀⣠⠾⠃⠀⠀⠀⣀⣴⣻⣟⡋⠉⠉⢻⠶⠀⠀⠀⠀⠀⠀
⠈⠑⠒⠒⠀⠀⢄⣀⡴⣯⣵⣖⣦⠤⣀⣀⣀⠉⠙⠒⠒⠒⠚⠉⢁⣀⣠⢤⣖⣿⣷⢯⡉⠉⠙⣲⠞⠁⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠈⠙⠣⢤⡞⠉⢉⡿⠒⢻⢿⡿⠭⣭⡭⠿⣿⡿⠒⠻⣯⡷⡄⠉⠳⣬⠷⠋⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠙⠺⠤⣄⣠⡏⠀⠀⡿⠀⠀⠘⡾⠀⢀⣈⡧⠴⠒⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠉⠙⠒⠓⠒⠒⠚⠛⠉⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"#,
        // Using image to ASCII converting
        _ => r#"
        .+:$X++.                        
        X&&&&&&X:    .;xxxxxx+++;;;::.  
       .x&$X&&xx. $xx&;;xxxx++;..       
     .+&&&&&&$&$. &.$xX&&&&&&&&x.       
         x&&&&&&; X:$;$&X&&&&&.         
       .$&&&&&&X; x.$;$&&&x.X&.         
       .&&X&&&$$  x:&;&&&&&&+ .         
        :&&&&&&..X:$:&&X$&;             
       .X$x&X&&X+x;x&&&&+.:             
       ;$&&&&&&&&X.;X$:                 
       :&&&&&&&&&&&&&&x:.    .X&&&&X;.  
       .$$+&+$&&&&&&&&&&&&&&$..x$;.     
     +;;.$&&&&&&&&&&&&&&&&&&&&+         
   .&x$X  .:::&&&X;+:::.::;$&&&&:       
     +&$X$&$:.X&x:    :&.    ;&&$       
     .&;...   ;Xx     x&.     x&&.      
     .+.      :&+      $&x.  :&&X       
          .+$$x&x:.    .x&&&&&$+        
         ..++&$x+$x.                    
           +;.                        "#,
    };

    println!("{}", RED);
    for line in ascii_art.lines() {
        println!("{}", line);
        thread::sleep(Duration::from_millis(100));
    }
    println!("{}", RESET);

    let propaganda = match rng.range(0, 5) {
        0 => "THEY ARE LYING TO YOU.",
        1 => "THE SKY WILL BURN FOR YOUR SINS.",
        2 => "SURRENDER IS SALVATION.",
        3 => "WE SEE EVERYTHING.",
        _ => "YOUR FAMILY IS NOT SAFE.",
    };

    print!("INTRUDER MESSAGE: ");
    io::stdout().flush().unwrap();
    print!("{}{}", RED, BOLD);
    print_slowly(propaganda, 150);
    print!("{}", RESET);
    
    thread::sleep(Duration::from_millis(800));
    println!("{}!!! SIGNAL TRACE FAILED. RESUMING NORMAL FEED. !!!{}", RED, RESET);
    thread::sleep(Duration::from_millis(500));
}

fn defcon_level(tension: f64) -> &'static str {
    if tension > 0.9 { "1 (IMMINENT NUCLEAR WAR)" }
    else if tension > 0.7 { "2 (NEXT STEP TO NUCLEAR WAR)" }
    else if tension > 0.5 { "3 (AIR FORCE READY TO MOBILIZE)" }
    else if tension > 0.3 { "4 (ABOVE NORMAL READINESS)" }
    else { "5 (NORMAL READINESS)" }
}

fn stability_desc(stability: f64) -> &'static str {
    if stability > 0.8 { "UNIFIED" }
    else if stability > 0.6 { "STABLE" }
    else if stability > 0.4 { "UNREST" }
    else if stability > 0.2 { "RIOTS" }
    else { "ANARCHY" }
}
