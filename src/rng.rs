use std::time::{SystemTime, UNIX_EPOCH};

pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new() -> Self {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let seed = since_the_epoch.as_nanos() as u64;
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        // Xorshift64*
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    pub fn next_f64(&mut self) -> f64 {
        // Generate float in [0, 1)
        (self.next_u64() as f64) / (u64::MAX as f64)
    }

    pub fn range(&mut self, min: u64, max: u64) -> u64 {
        if min >= max { return min; }
        min + (self.next_u64() % (max - min))
    }
    
    pub fn random_bool(&mut self, probability: f64) -> bool {
        self.next_f64() < probability
    }
}
