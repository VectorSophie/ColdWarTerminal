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
        // 1. Start Turn & Generate Docs (unless we just decrypted)
        if !skip_generation {
            engine.start_turn();
        } else {
            skip_generation = false; // Reset for next time
        }

        // 2. Display Status
        println!("\n{}--- TURN {} REPORT ---", CYAN, engine.turn_count);
        println!("DEFCON ESTIMATE: {}", defcon_level(engine.state.global_tension));
        println!("DOMESTIC MOOD:   {}{}", stability_desc(engine.state.domestic_stability), RESET);
        println!("{}----------------------{}", CYAN, RESET);

        // 3. Display Documents
        println!("\n{}INCOMING CABLES:{}", BOLD, RESET);
        for doc in &engine.pending_documents {
            // Interruption Logic
            if rng.random_bool(0.15) {
                trigger_interruption(&mut rng);
            }

            println!("\n{}[ID: {} | CLASS: {} | TIME: {}]{}", 
                CYAN, doc.id, doc.clearance_level, doc.timestamp, RESET);
            
            print!("> ");
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
        println!("\n{}AVAILABLE DIRECTIVES:{}", YELLOW, RESET);
        println!("1. ESCALATE    (Show force, risk war)");
        println!("2. INVESTIGATE (Internal audit, risk exposure)");
        println!("3. CONTAIN     (Diplomacy, look weak)");
        println!("4. LEAK        (Public transparency, chaos)");
        println!("5. STAND DOWN  (Withdraw, high political cost)");
        println!("6. DECRYPT [ID] (Analyze encrypted intel)");
        
        print!("\n{}AWAITING ORDER >> {}", GREEN, RESET);
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).expect("Failed to read input");
        let input = input.trim();
        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts.get(0).unwrap_or(&"").to_lowercase();
        let arg = parts.get(1).map(|s| s.to_string());

        let directive = match command.as_str() {
            "1" | "escalate" => Directive::Escalate,
            "2" | "investigate" => Directive::Investigate,
            "3" | "contain" => Directive::Contain,
            "4" | "leak" => Directive::Leak,
            "5" | "stand down" => Directive::StandDown,
            "6" | "decrypt" => {
                if let Some(id) = arg {
                    Directive::Decrypt(id)
                } else {
                    println!("{}ERROR: MISSING DOCUMENT ID (USAGE: DECRYPT DOC-XXXX){}", RED, RESET);
                    continue; // Skip the rest of loop, ask for input again
                }
            },
            "quit" | "exit" => break,
            _ => {
                println!("{}INVALID COMMAND. SYSTEM DEFAULTING TO 'CONTAIN'.{}", RED, RESET);
                Directive::Contain
            }
        };

        // If decrypting, we skip generating new docs next loop so player can act on info
        if let Directive::Decrypt(_) = directive {
            skip_generation = true;
        }

        // 5. Resolve
        let feedback = engine.resolve_directive(directive);
        println!("\n{}EXECUTING DIRECTIVE...{}", YELLOW, RESET);
        for line in feedback {
            if line.starts_with("CONTENT: ") {
                // SPECIAL ANIMATION FOR DECRYPTION
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

        if engine.turn_count >= 20 {
             println!("\n[SIMULATION END: MAX TURNS REACHED]");
             break;
        }
    }
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
    
    // Ensure display length matches target (scramble_text preserves length generally but good to be safe)
    if current_display.len() != len {
        current_display = vec!['#'; len];
    }

    // Iterate through each character position to "solve" it
    for i in 0..len {
        // Skip whitespace animation for speed, just lock it
        if target_chars[i].is_whitespace() {
            current_display[i] = ' ';
            continue;
        }

        // "Slot Machine" effect for the current character
        // Spin a few times
        for _ in 0..4 { 
            current_display[i] = random_char(rng);
            
            // Randomly flip some future characters too (Matrix rain style)
            // Only flip a few to avoid too much visual noise/lag
            let noise_idx = rng.range(i as u64, len as u64) as usize;
            if !target_chars[noise_idx].is_whitespace() {
                current_display[noise_idx] = random_char(rng);
            }

            // Print the line
            // \r returns cursor to start of line
            // We construct the string: 
            // GREEN (Solved part) + RED (Spinning char) + RED (Unsolved part)
            
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
            thread::sleep(Duration::from_millis(15)); // Fast spin
        }

        // Lock the correct character
        current_display[i] = target_chars[i];
        
        // Final print for this step (locked char becomes GREEN)
        let solved: String = current_display[0..=i].iter().collect();
        let unsolved: String = current_display[i+1..].iter().collect();
        print!("\r{}{}{}{}{}", GREEN, solved, RED, unsolved, RESET);
        io::stdout().flush().unwrap();
    }
    println!(); // Final newline
}

fn random_char(rng: &mut SimpleRng) -> char {
    // Hex-like + some symbols for a "digital cipher" look
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
    println!("\n{}!!! SIGNAL INTERRUPT DETECTED !!!{}", RED, RESET);
    thread::sleep(Duration::from_millis(500));
    
    let ascii_art = match rng.range(0, 3) {
        0 => r#"
   / \
  / ! \
 /_____\
        "#,
        1 => r#"
  (o) (o)
   \___/
        "#,
        _ => r#"
  [=====]
  |X . X|
  [=====]
        "#,
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
