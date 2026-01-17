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
        let id = format!("DOC-{:04X}", rng.range(0, 0xFFFF));
        
        // High chance for sensitive docs to be encrypted for gameplay demo purposes
        let mut is_encrypted = false;
        // Allow almost all docs to be encrypted/corrupted except public leaks
        if !matches!(doc_type, DocumentType::AnonymousLeak) {
            if rng.random_bool(0.7) {
                is_encrypted = true;
            }
        }

        let content = match doc_type {
            DocumentType::IntelligenceCable => generate_cable_content(state, rng, reliability),
            DocumentType::InternalMemo => generate_memo_content(state, rng, reliability),
            DocumentType::BudgetAnomaly => generate_budget_content(state, rng, reliability),
            DocumentType::ForeignIntercept => generate_intercept_content(state, rng, reliability),
            DocumentType::AnonymousLeak => generate_leak_content(state, rng, reliability),
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
