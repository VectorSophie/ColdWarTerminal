#[derive(Debug, Clone, PartialEq)]
pub enum AdvisorRole {
    General,
    Director,
    Ambassador,
}

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

#[derive(Debug, Clone)]
pub struct Advisor {
    pub name: String,
    pub role: AdvisorRole,
    pub suspicion: u32, // 0 to 100
    pub is_mole: bool,
    pub advice_log: Vec<(u32, String)>, // (turn_number, advice_text)
}

#[derive(Debug, Clone)]
pub struct WorldState {
    /// 0.0 (Peace) to 1.0 (Nuclear War)
    pub global_tension: f64,

    /// 0.0 (Open Society) to 1.0 (Totalitarian State)
    pub internal_secrecy: f64,

    /// 0.0 (Trusting) to 1.0 (Hostile)
    pub foreign_paranoia: f64,

    /// 0.0 (Safe) to 1.0 (Critical Failure Imminent)
    pub accidental_escalation_risk: f64,

    /// 0.0 (Anarchy) to 1.0 (Unified)
    pub domestic_stability: f64,

    /// Hidden internal weapon progress (0.0 to 1.0)
    pub secret_weapon_progress: f64,

    /// List of active advisors (one is a mole).
    pub advisors: Vec<Advisor>,
    /// Trigger for the "Red Phone" crisis event.
    pub red_phone_active: bool,
    /// 0.0 (Pure) to 1.0 (Corrupted) - affects system autonomy.
    pub system_corruption: f64,
}

impl WorldState {
    pub fn new() -> Self {
        // Initialize Advisors
        let advisors = vec![
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
        ];

        // Randomly assign one as the mole (logic will happen in game init since rng is there,
        // but for now we default to false and let GameEngine set it)

        Self {
            global_tension: 0.2,
            internal_secrecy: 0.5,
            foreign_paranoia: 0.3,
            accidental_escalation_risk: 0.05,
            domestic_stability: 0.8,
            secret_weapon_progress: 0.1,
            advisors,
            red_phone_active: false,
            system_corruption: 0.0,
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.global_tension >= 1.0
            || self.domestic_stability <= 0.0
            || self.system_corruption >= 1.0
    }
}
