use crate::state::WorldState;
use crate::document::Document;
use crate::rng::SimpleRng;

pub enum Directive {
    Escalate,
    Investigate,
    Contain,
    Leak,
    StandDown,
    Decrypt(String), // New Directive: Decrypt specific document ID
}

pub struct GameEngine {
    pub state: WorldState,
    pub turn_count: u32,
    pub pending_documents: Vec<Document>,
    rng: SimpleRng,
}

impl GameEngine {
    pub fn new() -> Self {
        Self {
            state: WorldState::new(),
            turn_count: 0,
            pending_documents: Vec::new(),
            rng: SimpleRng::new(),
        }
    }

    pub fn start_turn(&mut self) {
        self.turn_count += 1;
        let mut new_docs = Document::generate_batch(&self.state, 3);
        
        // GUARANTEE: Ensure at least one document is encrypted for the demo experience
        let has_encrypted = new_docs.iter().any(|d| d.is_encrypted);
        if !has_encrypted && !new_docs.is_empty() {
            new_docs[0].is_encrypted = true;
        }

        self.pending_documents = new_docs;
    }

    pub fn resolve_directive(&mut self, directive: Directive) -> Vec<String> {
        let mut feedback = Vec::new();

        match directive {
            Directive::Decrypt(target_id) => {
                // Find the doc
                let mut found = false;
                for doc in &mut self.pending_documents {
                    if doc.id == target_id {
                        if doc.is_encrypted {
                            doc.is_encrypted = false;
                            feedback.push(format!("SUCCESS: DOCUMENT {} DECRYPTED.", target_id));
                            feedback.push(format!("CONTENT: {}", doc.content));
                        } else {
                            feedback.push(format!("NOTICE: DOCUMENT {} WAS NOT ENCRYPTED.", target_id));
                        }
                        found = true;
                        break;
                    }
                }
                if !found {
                    feedback.push(format!("ERROR: DOCUMENT {} NOT FOUND IN CURRENT BATCH.", target_id));
                }
                // Decrypting consumes time; world tension drifts slightly
                if self.rng.random_bool(0.4) {
                    self.state.global_tension += 0.05;
                    feedback.push("While analysts worked, global tension drifted upward.".to_string());
                }
                
                // IMPORTANT: We do NOT advance the turn documents here, effectively allowing the player
                // to see the decrypted doc in the context of the SAME turn next loop? 
                // Actually, the main loop calls `start_turn` at the top. 
                // We need to signal to Main NOT to generate new docs if we just decrypted.
                // But for this simple architecture, let's say Decrypting uses the turn, and by the time you act next,
                // the situation has changed (new docs).
                // Wait, that defeats the purpose. You decrypt to KNOW what to do.
                // So Decrypt should likely be a "Free Action" or specific phase?
                // Let's make Decrypting take a turn, but we KEEP the old documents for the next turn 
                // so you can act on them.
                return feedback; 
            },
            Directive::Escalate => {
                if self.rng.random_bool(0.7) {
                    self.state.global_tension += 0.1;
                    self.state.foreign_paranoia += 0.15;
                    feedback.push("Directive executed: Military readiness increased.".to_string());
                    feedback.push("Intelligence reports enemy forces scrambling in response.".to_string());
                } else {
                    self.state.global_tension += 0.25;
                    self.state.accidental_escalation_risk += 0.1;
                    feedback.push("CRITICAL: Unit commander interpreted order as PREPARE TO FIRE. Stand down codes issued barely in time.".to_string());
                }
            },
            Directive::Investigate => {
                self.state.internal_secrecy -= 0.05;
                self.state.secret_weapon_progress += 0.05;
                feedback.push("Internal audit complete. Several discrepancies found in Sector 7.".to_string());
                if self.rng.random_bool(0.3) {
                     self.state.accidental_escalation_risk -= 0.05;
                     feedback.push("Safety protocols updated. Risk slightly mitigated.".to_string());
                }
            },
            Directive::Contain => {
                if self.state.foreign_paranoia > 0.7 {
                    feedback.push("Diplomatic channels ignored. Enemy is convinced we are stalling.".to_string());
                    self.state.global_tension += 0.05;
                } else {
                    self.state.global_tension -= 0.1;
                    self.state.domestic_stability -= 0.05;
                    feedback.push("Tension reduced via back-channel communications.".to_string());
                }
            },
            Directive::Leak => {
                self.state.internal_secrecy -= 0.2;
                self.state.domestic_stability += 0.1;
                self.state.foreign_paranoia -= 0.1;
                feedback.push("Information leaked to press. Public outrage is manageable, transparency increased.".to_string());
            },
            Directive::StandDown => {
                self.state.global_tension -= 0.3;
                self.state.foreign_paranoia -= 0.2;
                self.state.domestic_stability -= 0.2;
                 feedback.push("Forces withdrawn. Global sigh of relief.".to_string());
                 feedback.push("Generals are furious.".to_string());
            }
        }

        self.state.global_tension = self.state.global_tension.clamp(0.0, 1.0);
        self.state.internal_secrecy = self.state.internal_secrecy.clamp(0.0, 1.0);
        self.state.foreign_paranoia = self.state.foreign_paranoia.clamp(0.0, 1.0);
        self.state.accidental_escalation_risk = self.state.accidental_escalation_risk.clamp(0.0, 1.0);
        self.state.domestic_stability = self.state.domestic_stability.clamp(0.0, 1.0);
        self.state.secret_weapon_progress = self.state.secret_weapon_progress.clamp(0.0, 1.0);

        if self.state.accidental_escalation_risk > 0.5 && self.rng.random_bool(0.2) {
            self.state.global_tension += 0.1;
            feedback.push("WARNING: RADAR GLITCH DETECTED INCOMING MISSILES. DETERMINED FALSE ALARM.".to_string());
        }

        if self.state.secret_weapon_progress > 0.8 && self.rng.random_bool(0.1) {
             self.state.foreign_paranoia += 0.2;
             feedback.push(" INTELLIGENCE: ENEMY KNOWS ABOUT 'BASILISK'. THEY ARE PANICKING.".to_string());
        }

        feedback
    }
}
