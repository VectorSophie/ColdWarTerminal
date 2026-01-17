use crate::state::WorldState;
use crate::document::Document;
use crate::rng::SimpleRng;

#[derive(PartialEq)]
pub enum Directive {
    Escalate,
    Investigate,
    Contain,
    Leak,
    StandDown,
    Decrypt(String), // Minor Action (Costs 1 IP)
    Analyze(String), // Minor Action (Costs 1 IP)
}

pub struct GameEngine {
    pub state: WorldState,
    pub turn_count: u32,
    pub pending_documents: Vec<Document>,
    pub intel_points: u32,
    pub max_intel_points: u32,
    rng: SimpleRng,
}

impl GameEngine {
    pub fn new() -> Self {
        Self {
            state: WorldState::new(),
            turn_count: 0,
            pending_documents: Vec::new(),
            intel_points: 1,
            max_intel_points: 1,
            rng: SimpleRng::new(),
        }
    }

    pub fn start_turn(&mut self) {
        self.turn_count += 1;
        
        // SCALING DIFFICULTY: Cable Volume
        let doc_count = if self.turn_count >= 7 { 5 }
                       else if self.turn_count >= 4 { 4 }
                       else { 3 };

        // SCALING CAPABILITY: Intel Points
        self.max_intel_points = if self.turn_count >= 6 { 3 }
                               else if self.turn_count >= 3 { 2 }
                               else { 1 };
        self.intel_points = self.max_intel_points;

        let mut new_docs = Document::generate_batch(&self.state, doc_count);
        
        // GUARANTEE: Ensure at least one document is encrypted for the demo experience
        let has_encrypted = new_docs.iter().any(|d| d.is_encrypted);
        if !has_encrypted && !new_docs.is_empty() {
            new_docs[0].is_encrypted = true;
        }

        self.pending_documents = new_docs;
    }

    // Returns (Feedback Lines, Did Turn End?)
    pub fn resolve_directive(&mut self, directive: Directive) -> (Vec<String>, bool) {
        let mut feedback = Vec::new();
        let mut turn_ended = true;

        match directive {
            Directive::Decrypt(target_id) => {
                turn_ended = false;
                if self.intel_points == 0 {
                    feedback.push("FAILURE: INSUFFICIENT INTEL ASSETS. YOU MUST ACT NOW.".to_string());
                    return (feedback, false);
                }

                self.intel_points -= 1;
                let mut found = false;
                for doc in &mut self.pending_documents {
                    if doc.id == target_id {
                        if doc.is_encrypted {
                            doc.is_encrypted = false;
                            feedback.push(format!("SUCCESS: DOCUMENT {} DECRYPTED.", target_id));
                            feedback.push(format!("CONTENT: {}", doc.content));
                        } else {
                            feedback.push(format!("NOTICE: DOCUMENT {} WAS NOT ENCRYPTED. (Intel Asset Wasted)", target_id));
                        }
                        found = true;
                        break;
                    }
                }
                if !found {
                    feedback.push(format!("ERROR: DOCUMENT {} NOT FOUND.", target_id));
                    self.intel_points += 1; // Refund if typo
                }
            },
            Directive::Analyze(target_id) => {
                turn_ended = false;
                if self.intel_points == 0 {
                    feedback.push("FAILURE: INSUFFICIENT INTEL ASSETS. YOU MUST ACT NOW.".to_string());
                    return (feedback, false);
                }

                self.intel_points -= 1;
                let mut found = false;
                for doc in &self.pending_documents {
                    if doc.id == target_id {
                        let integrity = (doc.reliability * 100.0) as u32;
                        let assessment = if integrity > 80 { "HIGH (VERIFIED)" }
                                        else if integrity > 50 { "MODERATE (UNCERTAIN)" }
                                        else { "LOW (POSSIBLE DISINFORMATION)" };
                        
                        feedback.push(format!("ANALYSIS COMPLETE: DOCUMENT {}", target_id));
                        feedback.push(format!("SOURCE RELIABILITY: {}% - {}", integrity, assessment));
                        found = true;
                        break;
                    }
                }
                if !found {
                    feedback.push(format!("ERROR: DOCUMENT {} NOT FOUND.", target_id));
                    self.intel_points += 1; // Refund
                }
            },
            Directive::Escalate => {
                // ESCALATE: High Risk, High Reward
                if self.rng.random_bool(0.6) {
                    self.state.global_tension += 0.2;
                    self.state.foreign_paranoia += 0.2;
                    self.state.domestic_stability += 0.05;
                    feedback.push("Directive executed: GLOBAL STRIKE ASSETS PRIMED.".to_string());
                    feedback.push("Intelligence reports panic in enemy high command.".to_string());
                } else {
                    self.state.global_tension += 0.35;
                    self.state.accidental_escalation_risk += 0.15;
                    feedback.push("CRITICAL: MISCOMMUNICATION. SQUADRON LAUNCHED TACTICAL NUKE. ABORTED MID-FLIGHT.".to_string());
                }
            },
            Directive::Investigate => {
                self.state.internal_secrecy -= 0.1;
                self.state.secret_weapon_progress += 0.15;
                feedback.push("Internal audit reveals deeper layers of the Project.".to_string());
                if self.rng.random_bool(0.5) {
                     self.state.accidental_escalation_risk -= 0.1;
                     feedback.push("Protocols tightened. We are watching the watchers.".to_string());
                }
            },
            Directive::Contain => {
                if self.state.foreign_paranoia > 0.6 {
                    feedback.push("Diplomacy FAILED. Enemy interprets silence as preparation for war.".to_string());
                    self.state.global_tension += 0.1;
                } else {
                    self.state.global_tension -= 0.15;
                    self.state.domestic_stability -= 0.1;
                    feedback.push("Tension reduced. Military leadership questions your resolve.".to_string());
                }
            },
            Directive::Leak => {
                self.state.internal_secrecy -= 0.25;
                self.state.domestic_stability += 0.2;
                self.state.foreign_paranoia -= 0.05;
                feedback.push("The truth is out. The public riots, but they trust you more than the Generals.".to_string());
            },
            Directive::StandDown => {
                self.state.global_tension -= 0.4;
                self.state.foreign_paranoia -= 0.3;
                self.state.domestic_stability -= 0.35;
                 feedback.push("Total withdrawal ordered. We are naked before our enemies.".to_string());
                 feedback.push("Rumors of a military tribunal are circulating.".to_string());
            }
        }

        // Only apply passive effects if the turn actually ended
        if turn_ended {
            // PASSIVE ESCALATION
            if self.state.global_tension > 0.3 {
                self.state.global_tension += 0.03;
            }
            if self.state.secret_weapon_progress > 0.2 {
                 self.state.secret_weapon_progress += 0.02;
            }
            
            // Clamp values
            self.state.global_tension = self.state.global_tension.clamp(0.0, 1.0);
            self.state.internal_secrecy = self.state.internal_secrecy.clamp(0.0, 1.0);
            self.state.foreign_paranoia = self.state.foreign_paranoia.clamp(0.0, 1.0);
            self.state.accidental_escalation_risk = self.state.accidental_escalation_risk.clamp(0.0, 1.0);
            self.state.domestic_stability = self.state.domestic_stability.clamp(0.0, 1.0);
            self.state.secret_weapon_progress = self.state.secret_weapon_progress.clamp(0.0, 1.0);

            // Passive Effects
            if self.state.accidental_escalation_risk > 0.6 && self.rng.random_bool(0.3) {
                self.state.global_tension += 0.15;
                feedback.push("WARNING: UNAUTHORIZED SILO ACTIVATION DETECTED.".to_string());
            }

            if self.state.secret_weapon_progress > 0.9 && self.rng.random_bool(0.2) {
                 feedback.push(" THE BASILISK IS SPEAKING TO THE OPERATORS. THEY ARE WEEPING.".to_string());
            }
        }

        (feedback, turn_ended)
    }
}
