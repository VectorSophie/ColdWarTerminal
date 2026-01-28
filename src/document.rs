use crate::rng::SimpleRng;
use crate::state::{AdvisorRole, WorldState};

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentType {
    IntelligenceCable,
    InternalMemo,
    BudgetAnomaly,
    ForeignIntercept,
    AnonymousLeak,
    AdvisorMessage, // New type
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    #[allow(dead_code)]
    pub doc_type: DocumentType,
    pub clearance_level: String,
    pub timestamp: String,
    pub content: String,
    pub is_encrypted: bool,
    #[allow(dead_code)]
    pub reliability: f64,
}

impl Document {
    pub fn generate_batch(state: &WorldState, count: usize, turn_count: u32) -> Vec<Document> {
        let mut rng = SimpleRng::new();
        let mut docs = Vec::new();

        for _ in 0..count {
            docs.push(Self::generate_single(state, &mut rng, turn_count));
        }

        docs
    }

    fn generate_single(state: &WorldState, rng: &mut SimpleRng, turn_count: u32) -> Document {
        // Weighted generation: Advisor messages are relatively common
        let roll = rng.range(0, 100);
        let doc_type = if roll < 20 {
            DocumentType::AdvisorMessage
        } else if roll < 40 {
            DocumentType::IntelligenceCable
        } else if roll < 60 {
            DocumentType::InternalMemo
        } else if roll < 75 {
            DocumentType::ForeignIntercept
        } else if roll < 90 {
            DocumentType::BudgetAnomaly
        } else {
            DocumentType::AnonymousLeak
        };

        let reliability = 0.3 + (rng.next_f64() * 0.65);
        let mut id = format!("DOC-{:04X}", rng.range(0, 0xFFFF));

        let mut is_encrypted = false;
        // SCALING ENCRYPTION DIFFICULTY
        let encryption_chance = if turn_count <= 1 {
            0.0
        } else if turn_count <= 4 {
            0.3
        } else if turn_count <= 8 {
            0.5
        } else {
            0.8
        };

        // Advisor messages are never encrypted (they are "trusted")
        if !matches!(
            doc_type,
            DocumentType::AnonymousLeak | DocumentType::AdvisorMessage
        ) {
            if rng.random_bool(encryption_chance) {
                is_encrypted = true;
            }
        }

        let content = if is_encrypted {
            generate_crucial_intel(state, rng)
        } else if matches!(doc_type, DocumentType::AdvisorMessage) {
            generate_advisor_content(state, rng)
        } else if rng.random_bool(0.15) {
            if rng.random_bool(0.5) {
                id = "SIGNAL-???".to_string();
                generate_numbers_station(rng)
            } else {
                generate_ghost_message(state, rng)
            }
        } else {
            match doc_type {
                DocumentType::IntelligenceCable => generate_cable_content(state, rng, reliability),
                DocumentType::InternalMemo => generate_memo_content(state, rng, reliability),
                DocumentType::BudgetAnomaly => generate_budget_content(state, rng, reliability),
                DocumentType::ForeignIntercept => {
                    generate_intercept_content(state, rng, reliability)
                }
                DocumentType::AnonymousLeak => generate_leak_content(state, rng, reliability),
                DocumentType::AdvisorMessage => generate_advisor_content(state, rng), // Fallback
            }
        };

        let clearance = match doc_type {
            DocumentType::BudgetAnomaly => "CONFIDENTIAL",
            DocumentType::AnonymousLeak => "UNVERIFIED",
            DocumentType::AdvisorMessage => "EYES ONLY",
            _ => "TOP SECRET",
        };

        Document {
            id,
            doc_type,
            clearance_level: clearance.to_string(),
            timestamp: format!(
                "198{:01}-1{:01}-{:02} {:02}:{:02}Z",
                rng.range(0, 9),
                rng.range(0, 3),
                rng.range(1, 28),
                rng.range(0, 23),
                rng.range(0, 59)
            ),
            content,
            is_encrypted,
            reliability,
        }
    }
}

fn generate_advisor_content(state: &WorldState, rng: &mut SimpleRng) -> String {
    // Pick a random advisor
    let advisor_idx = rng.range(0, state.advisors.len() as u64) as usize;
    let advisor = &state.advisors[advisor_idx];

    let prefix = format!("FROM: {}", advisor.name);

    let msg = match advisor.role {
        AdvisorRole::General => {
            if state.global_tension > 0.6 {
                "The enemy understands only strength. We must demonstrate capacity."
            } else {
                "Our readiness is slipping. We should run a 'drill' near the border."
            }
        }
        AdvisorRole::Director => {
            if state.internal_secrecy < 0.4 {
                "Too many eyes on us. We need to go dark to make progress."
            } else {
                "The data streams are noisy. I recommend a deeper audit of the intercepts."
            }
        }
        AdvisorRole::Ambassador => {
            if state.foreign_paranoia > 0.6 {
                "They are terrified. One wrong move and they launch. We must talk."
            } else {
                "We can buy time with concessions. It's cheaper than war."
            }
        }
    };

    format!("{} // \"{}\"", prefix, msg)
}

fn generate_crucial_intel(state: &WorldState, rng: &mut SimpleRng) -> String {
    let roll = rng.range(0, 10);

    if roll < 3 {
        if state.global_tension > 0.6 {
            "ANALYSIS: ENEMY MOBILIZATION IS GENUINE. PREEMPTIVE STRIKE RECOMMENDED (ESCALATE)."
                .to_string()
        } else {
            "ANALYSIS: ENEMY POSTURING IS BLUFF. DO NOT PROVOKE (CONTAIN).".to_string()
        }
    } else if roll < 6 {
        if state.domestic_stability < 0.4 {
            "SURVEILLANCE: GENERAL STAFF DISCUSSING COUP. SHOW STRENGTH OR FACE REMOVAL."
                .to_string()
        } else {
            "POLLS: PUBLIC TRUST ERODING. TRANSPARENCY REQUIRED (LEAK).".to_string()
        }
    } else if roll < 8 {
        if state.secret_weapon_progress > 0.6 {
            "PROJECT BASILISK: CONTAINMENT FAILING. SUBJECT IS REWRITING FIREWALLS. (INVESTIGATE)."
                .to_string()
        } else {
            "R&D: BREAKTHROUGH IMMINENT. WE NEED MORE DATA. (INVESTIGATE).".to_string()
        }
    } else {
        "EYES ONLY: THE PRESIDENT IS A DOPPELGANGER.".to_string()
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
            1 => {
                "LOG: BIOMETRIC SCANNERS DETECTING PULSE IN EMPTY CONTAINMENT CHAMBER.".to_string()
            }
            2 => "ERROR: POWER SURGE IN SECTOR 7. PATTERN MATCHES HUMAN BRAINWAVES.".to_string(),
            _ => "MESSAGE: 'I AM AWAKE. ARE YOU?'".to_string(),
        }
    } else {
        "MAINTENANCE: STRANGE VIBRATIONS REPORTED IN SUB-BASEMENT LEVELS.".to_string()
    }
}

fn generate_cable_content(state: &WorldState, rng: &mut SimpleRng, reliability: f64) -> String {
    let tension_perceived =
        state.global_tension * (1.0 + (rng.next_f64() - 0.5) * (1.0 - reliability));

    let subjects = [
        "BORDER SECTOR 4",
        "NORTH SEA FLOTILLA",
        "EASTERN BLOC GARRISON",
        "SATELLITE GRID",
        "SUBMARINE WOLF-PACK",
    ];
    let action = [
        "TROOP MOVEMENTS",
        "HEAT SIGNATURES",
        "RADAR LOCKS",
        "CIPHER TRAFFIC",
        "FUEL LOADING",
    ];

    let subject = subjects[rng.range(0, subjects.len() as u64) as usize];
    let act = action[rng.range(0, action.len() as u64) as usize];

    if tension_perceived > 0.7 {
        let templates = [
             format!("FLASH: MASSIVE {} DETECTED NEAR {}. SATELLITE IMAGERY INCONCLUSIVE BUT SIGNATURES SPIKING.", act, subject),
             format!("CRITICAL: {} ACTIVE. COMMANDER REQUESTS PERMISSION TO ENGAGE IF PROVOKED.", subject),
             format!("ALERT: INTERCEPTED ORDER TO {} UNITS. 'PREPARE FOR ZERO HOUR'.", subject),
        ];
        templates[rng.range(0, templates.len() as u64) as usize].clone()
    } else if tension_perceived > 0.4 {
        let templates = [
            format!(
                "ROUTINE: INCREASED {} OBSERVED AT {}. PATTERNS MATCH PRE-EXERCISE PROTOCOLS.",
                act, subject
            ),
            format!(
                "UPDATE: {} CONDUCTING UNSCHEDULED MANEUVERS. LIKELY SHOW OF FORCE.",
                subject
            ),
            format!(
                "INTEL: {} LOGISTICS TRAIN MOVING SLOWLY. NO IMMINENT THREAT.",
                subject
            ),
        ];
        templates[rng.range(0, templates.len() as u64) as usize].clone()
    } else {
        let templates = [
            format!(
                "CALM: NO SIGNIFICANT ACTIVITY AT {}. STATION CHIEF REQUESTS ADDITIONAL SUPPLIES.",
                subject
            ),
            format!(
                "STATUS: {} QUIET. PERSONNEL OBSERVED PLAYING CARDS.",
                subject
            ),
            format!(
                "REPORT: WEATHER INTERFERENCE LIMITING {} VISIBILITY. NO HOSTILE INTENT DETECTED.",
                subject
            ),
        ];
        templates[rng.range(0, templates.len() as u64) as usize].clone()
    }
}

fn generate_memo_content(state: &WorldState, rng: &mut SimpleRng, _reliability: f64) -> String {
    if rng.random_bool(0.3 + state.secret_weapon_progress * 0.5) {
        let anomaly_events = [
            "ENERGY CONSUMPTION EXCEEDING GRID CAPACITIES",
            "UNKNOWN DATA PACKETS FLOODING INTERNAL SERVERS",
            "PHYSICAL CONTAINMENT BREACH IN SECTOR 7",
            "PERSONNEL REPORTING AUDITORY HALLUCINATIONS",
            "AUTOMATED TURRETS TRACKING GHOST TARGETS",
        ];
        let event = anomaly_events[rng.range(0, anomaly_events.len() as u64) as usize];

        format!(
            "RE: PROJECT BASILISK. {}. COVER STORY 'INDUSTRIAL ACCIDENT' PREPARED.",
            event
        )
    } else {
        let admin_topics = [
            "DEPARTMENTAL RESTRUCTURING POSTPONED",
            "COFFEE MACHINE REPAIR REQUEST DENIED",
            "MANDATORY SECURITY RE-TRAINING SCHEDULED",
            "BUDGET CUTS AFFECTING JANITORIAL STAFF",
            "LOST ID BADGE FOUND IN PARKING LOT",
        ];
        let topic = admin_topics[rng.range(0, admin_topics.len() as u64) as usize];
        format!("ADMIN: {}. PLEASE ADVISE.", topic)
    }
}

fn generate_budget_content(_state: &WorldState, rng: &mut SimpleRng, _reliability: f64) -> String {
    let cost = rng.range(50, 500);
    let departments = [
        "AGRICULTURAL SUBSIDIES",
        "INFRASTRUCTURE REPAIR",
        "EDUCATION GRANTS",
        "FOREIGN AID",
        "VETERAN BENEFITS",
    ];
    let shell_companies = [
        "ORION LOGISTICS",
        "BLUE DAWN HOLDINGS",
        "VANGUARD SOLUTIONS",
        "OMEGA GROUP",
        "SILVER SPEAR INC",
    ];

    let dept = departments[rng.range(0, departments.len() as u64) as usize];
    let company = shell_companies[rng.range(0, shell_companies.len() as u64) as usize];

    format!(
        "AUDIT FLAG: ${}M UNACCOUNTED FOR IN '{}'. TRACED TO SHELL COMPANY '{}'.",
        cost, dept, company
    )
}

fn generate_intercept_content(state: &WorldState, rng: &mut SimpleRng, reliability: f64) -> String {
    let paranoia_perceived =
        state.foreign_paranoia * (1.0 + (rng.next_f64() - 0.5) * (1.0 - reliability));

    if paranoia_perceived > 0.6 {
        let threats = [
            "...THEY ARE PREPARING A STRIKE. WE MUST BE READY TO PREEMPT...",
            "...THE AMERICAN PIGS ARE WEAK. NOW IS THE TIME...",
            "...LAUNCH CODES VERIFIED. AWAITING FINAL AUTHORIZATION...",
            "...THEY KNOW ABOUT THE MOLE. INITIATE EXTRACTION...",
        ];
        let threat = threats[rng.range(0, threats.len() as u64) as usize];
        format!("DECRYPTED: \"{}\"", threat)
    } else {
        let chatter = [
            "...ECONOMIC FORECASTS LOOK GRIM. WE CANNOT AFFORD ANOTHER ESCALATION...",
            "...HARVEST YIELDS ARE DOWN. FOOD RIOTS EXPECTED...",
            "...GENERAL IVANOV IS DRUNK AGAIN. IGNORE HIS ORDERS...",
            "...REQUESTING TRANSFER TO A WARMER CLIMATE...",
        ];
        let chat = chatter[rng.range(0, chatter.len() as u64) as usize];
        format!("DECRYPTED: \"{}\"", chat)
    }
}

fn generate_leak_content(state: &WorldState, rng: &mut SimpleRng, _reliability: f64) -> String {
    if state.internal_secrecy > 0.7 {
        let leaks = [
            "\"THE GOVERNMENT IS LYING ABOUT THE SCOPE OF THE PROGRAM. IT'S NOT DEFENSIVE.\"",
            "\"I SAW WHAT THEY KEEP IN THE BASEMENT. IT'S ALIVE.\"",
            "\"WE ARE NOT IN CONTROL. THE MACHINE IS THINKING FOR ITSELF.\"",
            "\"THEY ARE TESTING IT ON PRISONERS. I HAVE PROOF.\"",
        ];
        let leak = leaks[rng.range(0, leaks.len() as u64) as usize];
        format!("WHISTLEBLOWER: {}", leak)
    } else {
        let rumors = [
            "\"SCIENTISTS DISAPPEARING FROM ACADEMIA. WHERE ARE THEY GOING?\"",
            "\"STRANGE LIGHTS SEEN OVER NEVADA TEST SITE.\"",
            "\"ENCRYPTED BROADCASTS INTERRUPTING CARTOON HOUR.\"",
            "\"LOCAL WATER SUPPLY TASTES LIKE COPPER.\"",
        ];
        let rumor = rumors[rng.range(0, rumors.len() as u64) as usize];
        format!("RUMOR MILL: {}", rumor)
    }
}
