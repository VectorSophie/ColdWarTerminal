#[derive(Debug, Clone)]
pub struct WorldState {
    /// 0.0 (Peace) to 1.0 (Nuclear War)
    pub global_tension: f64,
    
    /// 0.0 (Open Society) to 1.0 (Totalitarian State)
    /// High secrecy increases difficultly of parsing truth but protects against leaks.
    pub internal_secrecy: f64,
    
    /// 0.0 (Trusting) to 1.0 (Hostile)
    /// How likely the enemy is to interpret actions as aggression.
    pub foreign_paranoia: f64,
    
    /// 0.0 (Safe) to 1.0 (Critical Failure Imminent)
    /// Represents accumulated system errors, fatigue, and technical glitches.
    pub accidental_escalation_risk: f64,

    /// 0.0 (Anarchy) to 1.0 (Unified)
    /// If this drops too low, you lose via coup/collapse.
    pub domestic_stability: f64,
    
    /// Hidden internal weapon progress (0.0 to 1.0)
    /// The "Thing" the documents hint at.
    pub secret_weapon_progress: f64,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            global_tension: 0.2,
            internal_secrecy: 0.5,
            foreign_paranoia: 0.3,
            accidental_escalation_risk: 0.05,
            domestic_stability: 0.8,
            secret_weapon_progress: 0.1,
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.global_tension >= 1.0 || self.domestic_stability <= 0.0
    }
}
