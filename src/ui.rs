use crate::rng::SimpleRng;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

// --- COLORS (Extended ANSI) ---
pub const TEAL: &str = "\x1b[38;5;14m";
pub const AMBER: &str = "\x1b[38;5;214m";
pub const ORANGE: &str = "\x1b[38;5;202m";
pub const RED_ALERT: &str = "\x1b[38;5;196m";
pub const GREY_DIM: &str = "\x1b[38;5;240m";
pub const WHITE_BOLD: &str = "\x1b[1;37m";
pub const RESET: &str = "\x1b[0m";

// --- SYMBOLS ---
const H_LINE: char = '─';
const V_LINE: char = '│';
const TL_CORNER: char = '┌';
const TR_CORNER: char = '┐';
const BL_CORNER: char = '└';
const BR_CORNER: char = '┘';
const BLOCK_STATUS_1: char = '█';

const BLOCK_STATUS_2: char = '▒';
const BLOCK_STATUS_3: char = '░';

/// Clears the terminal screen and moves cursor to top-left.
pub fn clear_screen() {
    print!("\x1b[2J\x1b[1;1H");
}

/// Renders a "glitched" progress bar.
pub fn draw_progress_bar(label: &str, value: f64, width: usize, color: &str, rng: &mut SimpleRng) {
    let bar_width = width - label.len() - 8; // -8 for brackets and percentage
    let filled = (value * bar_width as f64).round() as usize;
    let empty = bar_width.saturating_sub(filled);

    print!("{:<15} [", label);
    print!("{}", color);

    for _i in 0..filled {
        // Occasional glitch in the bar
        if rng.random_bool(0.05) {
            print!("{}", BLOCK_STATUS_2);
        } else {
            print!("{}", BLOCK_STATUS_1);
        }
    }

    print!("{}", GREY_DIM);
    for _ in 0..empty {
        print!("{}", BLOCK_STATUS_3);
    }

    print!("{}]{} {:>3}%", RESET, color, (value * 100.0) as u32);
    println!("{}", RESET);
}

/// Prints text with a typewriter effect, optionally glitching characters.
pub fn type_text(text: &str, speed_ms: u64, color: &str, glitch_chance: f64, rng: &mut SimpleRng) {
    print!("{}", color);
    for c in text.chars() {
        if glitch_chance > 0.0 && rng.random_bool(glitch_chance) {
            let glitch_char = (rng.range(33, 126) as u8) as char;
            print!("{}", glitch_char);
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(20));
            print!("\x08"); // Backspace
        }
        print!("{}", c);
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(speed_ms));
    }
    println!("{}", RESET);
}

/// Draws the main HUD header.
pub fn draw_hud(turn: u32, tension: f64, intel: u32, max_intel: u32) {
    let width = 60;
    let inner_width = width - 2;

    let date_str = format!("DAY {:03} // 1983", turn);
    let intel_str = format!("INTEL: {}/{}", intel, max_intel);
    let defcon_plain_str = format!("DEFCON: {:.2}", tension);

    // Calculate dynamic spacing
    // We have 3 items: [date] [defcon] [intel]
    // Total content length
    let content_len = date_str.len() + defcon_plain_str.len() + intel_str.len();

    // Check if we have space (we should, ~37 chars vs 58 space)
    let available_space = if content_len < inner_width {
        inner_width - content_len
    } else {
        0
    };

    // Distribute space:
    // Left padding: 1 (if possible)
    // Gap 1 (Date->Defcon): remaining / 2
    // Gap 2 (Defcon->Intel): remaining - Gap 1
    // Right padding: 1 (if possible) - actually included in gaps usually or just ensure spacing.

    // Let's go for specific look:
    // | DAY...   DEFCON...   INTEL... |
    // We want at least 1 space between items.

    // Simple distribution:
    // [Date] [Gap1] [Defcon] [Gap2] [Intel]
    // We won't put padding on far left/right edges to maximize internal spacing,
    // or we can put 1 space left/right for aesthetics.
    // Let's put 1 space left and 1 space right if we have enough space.

    let (pad_left, pad_right, gap1, gap2) = if available_space >= 4 {
        let internal_space = available_space - 2; // Reserve 1 left, 1 right
        let g1 = internal_space / 2;
        let g2 = internal_space - g1;
        (1, 1, g1, g2)
    } else {
        // Not enough space for nice padding, just split between items
        let g1 = available_space / 2;
        let g2 = available_space - g1;
        (0, 0, g1, g2)
    };

    // Top Border
    println!(
        "{}{}{}{}",
        TEAL,
        TL_CORNER,
        H_LINE.to_string().repeat(inner_width),
        TR_CORNER
    );

    // Info Line construction
    let tension_color = if tension > 0.8 {
        RED_ALERT
    } else if tension > 0.5 {
        ORANGE
    } else {
        TEAL
    };

    print!("{}{}", TEAL, V_LINE); // Start border

    // Content
    print!("{}{}", " ".repeat(pad_left), date_str);
    print!("{}", " ".repeat(gap1));
    print!("DEFCON: {}{:.2}{}", tension_color, tension, TEAL); // Manual print to handle color
    print!("{}", " ".repeat(gap2));
    print!("{}{}", intel_str, " ".repeat(pad_right));

    println!("{}{}{}", TEAL, V_LINE, RESET); // End border

    // Bottom Border
    println!(
        "{}{}{}{}{}",
        TEAL,
        BL_CORNER,
        H_LINE.to_string().repeat(inner_width),
        BR_CORNER,
        RESET
    );
}
