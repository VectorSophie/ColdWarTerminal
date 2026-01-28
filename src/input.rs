use std::io;
use std::sync::mpsc;
use std::thread;

pub struct InputManager {
    rx: mpsc::Receiver<String>,
}

impl InputManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let stdin = io::stdin();
            loop {
                let mut buffer = String::new();
                if stdin.read_line(&mut buffer).is_ok() {
                    // We successfully read a line
                    if tx.send(buffer).is_err() {
                        break; // Receiver dropped
                    }
                }
            }
        });
        Self { rx }
    }

    /// Blocking read for the next line of input.
    pub fn read_line(&self) -> String {
        self.rx.recv().unwrap_or_default()
    }

    /// Clears any buffered input (useful before prompts)
    pub fn flush(&self) {
        while self.rx.try_recv().is_ok() {}
    }
}
