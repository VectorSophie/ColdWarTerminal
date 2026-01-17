use crate::state::WorldState;
use crate::rng::SimpleRng;

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentType {
    IntelligenceCable,
    InternalMemo,
    BudgetAnomaly,
    ForeignIntercept,
    AnonymousLeak,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    #[allow(dead_code)]
    pub doc_type: DocumentType,
    pub clearance_level: String,
    pub timestamp: String,
    pub content: String,
    pub is_encrypted: bool,     // New field
    #[allow(dead_code)]
    pub reliability: f64,
}

impl Document {
    pub fn generate_batch(state: &WorldState, count: usize) -> Vec<Document> {
        let mut rng = SimpleRng::new();
        let mut docs = Vec::new();

        for _ in 0..count {
            docs.push(Self::generate_single(state, &mut rng));
        }

        docs
    }

    fn generate_single(state: &WorldState, rng: &mut SimpleRng) -> Document {
        let doc_type = match rng.range(0, 5) {
            0 => DocumentType::IntelligenceCable,
            1 => DocumentType::InternalMemo,
            2 => DocumentType::BudgetAnomaly,
            3 => DocumentType::ForeignIntercept,
            _ => DocumentType::AnonymousLeak,
        };

        let reliability = 0.3 + (rng.next_f64() * 0.65);
        let mut id = format!("DOC-{:04X}", rng.range(0, 0xFFFF));
        
        // 50% chance for sensitive docs to be encrypted (High Value Intel)
        let mut is_encrypted = false;
        if !matches!(doc_type, DocumentType::AnonymousLeak) {
            if rng.random_bool(0.5) {
                is_encrypted = true;
            }
        }

        // CONTENT GENERATION
        let content = if is_encrypted {
            // Encrypted docs ALWAYS contain crucial, game-changing intel
            generate_crucial_intel(state, rng)
        } else if rng.random_bool(0.15) {
             // Chaos factor (Ghost/Numbers) - unencrypted anomalies
             if rng.random_bool(0.5) {
                 id = "SIGNAL-???".to_string();
                 generate_numbers_station(rng)
             } else {
                 generate_ghost_message(state, rng)
             }
        } else {
            // Standard fluff
            match doc_type {
                DocumentType::IntelligenceCable => generate_cable_content(state, rng, reliability),
                DocumentType::InternalMemo => generate_memo_content(state, rng, reliability),
                DocumentType::BudgetAnomaly => generate_budget_content(state, rng, reliability),
                DocumentType::ForeignIntercept => generate_intercept_content(state, rng, reliability),
                DocumentType::AnonymousLeak => generate_leak_content(state, rng, reliability),
            }
        };

        let clearance = match doc_type {
            DocumentType::BudgetAnomaly => "CONFIDENTIAL",
            DocumentType::AnonymousLeak => "UNVERIFIED",
            _ => "TOP SECRET",
        };

        Document {
            id,
            doc_type,
            clearance_level: clearance.to_string(),
            timestamp: format!("198{:01}-1{:01}-{:02} {:02}:{:02}Z", 
                rng.range(0, 9), rng.range(0, 3), rng.range(1, 28),
                rng.range(0, 23), rng.range(0, 59)),
            content,
            is_encrypted,
            reliability,
        }
    }
}

fn generate_crucial_intel(state: &WorldState, rng: &mut SimpleRng) -> String {
    // These messages hint at the 'correct' action or reveal hidden stat thresholds
    let roll = rng.range(0, 10);
    
    if roll < 3 {
        // TENSION INTEL
        if state.global_tension > 0.6 {
            "ANALYSIS: ENEMY MOBILIZATION IS GENUINE. PREEMPTIVE STRIKE RECOMMENDED (ESCALATE)." .to_string()
        } else {
             "ANALYSIS: ENEMY POSTURING IS BLUFF. DO NOT PROVOKE (CONTAIN)." .to_string()
        }
    } else if roll < 6 {
        // STABILITY INTEL
        if state.domestic_stability < 0.4 {
             "SURVEILLANCE: GENERAL STAFF DISCUSSING COUP. SHOW STRENGTH OR FACE REMOVAL." .to_string()
        } else {
             "POLLS: PUBLIC TRUST ERODING. TRANSPARENCY REQUIRED (LEAK)." .to_string()
        }
    } else if roll < 8 {
        // BASILISK INTEL
        if state.secret_weapon_progress > 0.6 {
             "PROJECT BASILISK: CONTAINMENT FAILING. SUBJECT IS REWRITING FIREWALLS. (INVESTIGATE)." .to_string()
        } else {
             "R&D: BREAKTHROUGH IMMINENT. WE NEED MORE DATA. (INVESTIGATE)." .to_string()
        }
    } else {
        // WILDCARD
        "EYES ONLY: THE PRESIDENT IS A DOPPELGANGER." .to_string()
    }
}

fn generate_numbers_station(rng: &mut SimpleRng) -> String {
    let mut s = "BROADCAST DETECTED: ".to_string();
    for _ in 0..6 {
        s.push_str(&format!("{:02} ", rng.range(0, 99)));
    }
    s.push_str("... [REPEATING]");
    s
}

fn generate_ghost_message(state: &WorldState, rng: &mut SimpleRng) -> String {
    if state.secret_weapon_progress > 0.5 {
        match rng.range(0, 4) {
            0 => "SYSTEM ALERT: UNKNOWN PROCESS 'BASILISK' REQUESTING ROOT ACCESS.".to_string(),
            1 => "LOG: BIOMETRIC SCANNERS DETECTING PULSE IN EMPTY CONTAINMENT CHAMBER.".to_string(),
            2 => "ERROR: POWER SURGE IN SECTOR 7. PATTERN MATCHES HUMAN BRAINWAVES.".to_string(),
            _ => "MESSAGE: 'I AM AWAKE. ARE YOU?'".to_string(),
        }
    } else {
        "MAINTENANCE: STRANGE VIBRATIONS REPORTED IN SUB-BASEMENT LEVELS.".to_string()
    }
}

fn generate_cable_content(state: &WorldState, rng: &mut SimpleRng, reliability: f64) -> String {
    let tension_perceived = state.global_tension * (1.0 + (rng.next_f64() - 0.5) * (1.0 - reliability));
    
    if tension_perceived > 0.7 {
        "FLASH: MASSIVE TROOP MOVEMENTS DETECTED NEAR BORDER SECTOR 4. SATELLITE IMAGERY INCONCLUSIVE BUT HEAT SIGNATURES SPIKING.".to_string()
    } else if tension_perceived > 0.4 {
        "ROUTINE: INCREASED RADIO CHATTER OBSERVED. PATTERNS MATCH PRE-EXERCISE PROTOCOLS.".to_string()
    } else {
        "CALM: NO SIGNIFICANT ACTIVITY TO REPORT. STATION CHIEF REQUESTS ADDITIONAL SUPPLIES.".to_string()
    }
}

fn generate_memo_content(state: &WorldState, rng: &mut SimpleRng, _reliability: f64) -> String {
    if rng.random_bool(0.3 + state.secret_weapon_progress * 0.5) {
        "RE: PROJECT BASILISK. ENERGY CONSUMPTION EXCEEDING GRID CAPACITIES IN SECTOR 7. COVER STORY 'INDUSTRIAL ACCIDENT' PREPARED.".to_string()
    } else {
        "ADMIN: DEPARTMENTAL RESTRUCTURING POSTPONED DUE TO SECURITY CONCERNS.".to_string()
    }
}

fn generate_budget_content(_state: &WorldState, rng: &mut SimpleRng, _reliability: f64) -> String {
     let cost = rng.range(50, 500);
     format!("AUDIT FLAG: ${}M UNACCOUNTED FOR IN 'AGRICULTURAL SUBSIDIES'. TRACED TO SHELL COMPANY 'ORION LOGISTICS'.", cost)
}

fn generate_intercept_content(state: &WorldState, rng: &mut SimpleRng, reliability: f64) -> String {
    let paranoia_perceived = state.foreign_paranoia * (1.0 + (rng.next_f64() - 0.5) * (1.0 - reliability));
    
    if paranoia_perceived > 0.6 {
        "DECRYPTED: \"...THEY ARE PREPARING A STRIKE. WE MUST BE READY TO PREEMPT. THE SILOS ARE OPENING...\"".to_string()
    } else {
        "DECRYPTED: \"...ECONOMIC FORECASTS LOOK GRIM. WE CANNOT AFFORD ANOTHER ESCALATION...\"".to_string()
    }
}

fn generate_leak_content(state: &WorldState, _rng: &mut SimpleRng, _reliability: f64) -> String {
    if state.internal_secrecy > 0.7 {
        "WHISTLEBLOWER: \"THE GOVERNMENT IS LYING ABOUT THE SCOPE OF THE PROGRAM. IT'S NOT DEFENSIVE.\"".to_string()
    } else {
        "RUMOR MILL: \"SCIENTISTS DISAPPEARING FROM ACADEMIA. WHERE ARE THEY GOING?\"" .to_string()
    }
}
