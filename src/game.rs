use crate::document::Document;
use crate::rng::SimpleRng;
use crate::state::{AdvisorRole, WorldState};

#[derive(PartialEq)]
pub enum Directive {
    Escalate,
    Investigate,
    Contain,
    Leak,
    StandDown,
    Decrypt(String),
    Analyze(String),
    Trace,
    Consult(String), // New Directive
}

pub struct GameEngine {
    pub state: WorldState,
    pub turn_count: u32,
    pub pending_documents: Vec<Document>,
    pub intel_points: u32,
    pub max_intel_points: u32,
    pub interruption_active: bool,
    pub consult_count: u32, // Track consults per turn
    rng: SimpleRng,
}

impl GameEngine {
    pub fn new() -> Self {
        let mut rng = SimpleRng::new();
        let mut state = WorldState::new();

        // Assign a random mole
        let mole_idx = rng.range(0, 3) as usize;
        state.advisors[mole_idx].is_mole = true;

        Self {
            state,
            turn_count: 0,
            pending_documents: Vec::new(),
            intel_points: 1,
            max_intel_points: 1,
            interruption_active: false,
            consult_count: 0,
            rng,
        }
    }

    pub fn start_turn(&mut self) {
        self.turn_count += 1;
        self.interruption_active = false;
        self.consult_count = 0; // Reset consults

        // SCALING INTERRUPTION DIFFICULTY
        // Turn 1-2: 0%, Turn 3-5: 15%, Turn 6-10: 30%, Turn 11+: 50%
        let interruption_chance = if self.turn_count <= 2 {
            0.0
        } else if self.turn_count <= 5 {
            0.15
        } else if self.turn_count <= 10 {
            0.30
        } else {
            0.50
        };

        if self.rng.random_bool(interruption_chance) {
            self.interruption_active = true;
        }

        let doc_count = if self.turn_count >= 7 {
            5
        } else if self.turn_count >= 4 {
            4
        } else {
            3
        };

        self.max_intel_points = if self.turn_count >= 6 {
            3
        } else if self.turn_count >= 3 {
            2
        } else {
            1
        };
        self.intel_points = self.max_intel_points;

        let mut new_docs = Document::generate_batch(&self.state, doc_count, self.turn_count);

        let has_encrypted = new_docs.iter().any(|d| d.is_encrypted);
        if !has_encrypted && !new_docs.is_empty() {
            new_docs[0].is_encrypted = true;
        }

        self.pending_documents = new_docs;
    }

    pub fn resolve_directive(&mut self, directive: Directive) -> (Vec<String>, bool) {
        let mut feedback = Vec::new();
        let mut turn_ended = true;

        match directive {
            Directive::Trace => {
                turn_ended = false;
                if self.intel_points == 0 {
                    feedback.push("FAILURE: INSUFFICIENT INTEL ASSETS.".to_string());
                    return (feedback, false);
                }
                self.intel_points -= 1;

                if self.interruption_active {
                    feedback.push("TRACE INITIATED... SIGNAL LOCK ESTABLISHED.".to_string());
                    feedback.push("ORIGIN POINT TRIANGULATED:".to_string());

                    // Find the mole and increase suspicion
                    let mole_idx = self.state.advisors.iter().position(|a| a.is_mole).unwrap();
                    let mole_name = self.state.advisors[mole_idx].name.clone();

                    // Clue logic: Give a hint about the mole's role/name
                    if self.rng.random_bool(0.5) {
                        feedback.push(format!(
                            ">> PARTIAL MATCH: AUTHORIZED DEVICE REGISTERED TO '{}'.",
                            mole_name
                        ));
                    } else {
                        let role_str = match self.state.advisors[mole_idx].role {
                            AdvisorRole::General => "MILITARY COMMAND NODE",
                            AdvisorRole::Director => "INTELLIGENCE DATACENTER",
                            AdvisorRole::Ambassador => "DIPLOMATIC SECURE LINE",
                        };
                        feedback.push(format!(">> ROUTING DETECTED VIA {}.", role_str));
                    }

                    self.state.advisors[mole_idx].suspicion += 35;
                    if self.state.advisors[mole_idx].suspicion >= 100 {
                        // Trigger the Reveal/Crisis immediately?
                        // Or just note it. Let's note it.
                        feedback.push(
                            "!!! MOLE IDENTITY CONFIRMED. THEY KNOW WE KNOW. !!!".to_string(),
                        );
                        self.state.red_phone_active = true;
                    }
                } else {
                    feedback.push(
                        "TRACE FAILED: NO ACTIVE SIGNAL INTERRUPTION TO LOCK ONTO.".to_string(),
                    );
                    self.intel_points += 1; // Refund
                }
            }
            Directive::Consult(target) => {
                turn_ended = false;

                // Cost Logic: First one is free, subsequent cost 1 Intel
                if self.consult_count > 0 {
                    if self.intel_points == 0 {
                        feedback.push(
                            "FAILURE: INSUFFICIENT INTEL ASSETS FOR ADDITIONAL CONSULTATION."
                                .to_string(),
                        );
                        return (feedback, false);
                    }
                    self.intel_points -= 1;
                }
                self.consult_count += 1;

                // Find Advisor
                let target_lower = target.to_lowercase();
                let advisor = self.state.advisors.iter().find(|a| {
                    a.name.to_lowercase().contains(&target_lower)
                        || format!("{:?}", a.role)
                            .to_lowercase()
                            .contains(&target_lower)
                });

                if let Some(adv) = advisor {
                    let cost_msg = if self.consult_count > 1 {
                        "(INTEL COST: 1)"
                    } else {
                        "(STANDARD PROTOCOL)"
                    };
                    feedback.push(format!(
                        "CONSULTING WITH {}... {}",
                        adv.name.to_uppercase(),
                        cost_msg
                    ));

                    let advice = if adv.is_mole {
                        // Mole Logic: Mislead
                        match adv.role {
                            AdvisorRole::General => {
                                if self.state.global_tension > 0.7 {
                                    // Mole wants war: push for escalation when dangerous
                                    "We have the advantage! Strike now before they mobilize further! (Recommend: ESCALATE)".to_string()
                                } else {
                                    // Mole wants weakness: stand down when you should be alert
                                    "Intelligence is flawed. They are just exercises. We should pull back. (Recommend: STAND DOWN)".to_string()
                                }
                            }
                            AdvisorRole::Director => {
                                // Mole wants chaos/exposure
                                if self.state.internal_secrecy < 0.4 {
                                    "The leaks are useful. They confuse the enemy. Let them flow. (Recommend: LEAK)".to_string()
                                } else {
                                    "Our own agents are the problem. Purge the departments. (Recommend: INVESTIGATE)".to_string()
                                }
                            }
                            AdvisorRole::Ambassador => {
                                // Mole wants capitulation or mixed signals
                                if self.state.foreign_paranoia > 0.6 {
                                    "They are bluffing. Ignore their threats. (Recommend: CONTAIN)"
                                        .to_string()
                                } else {
                                    "We should apologize for the border incident immediately. (Recommend: STAND DOWN)".to_string()
                                }
                            }
                        }
                    } else {
                        // Loyal Logic: Sound advice
                        match adv.role {
                            AdvisorRole::General => {
                                if self.state.global_tension > 0.8 {
                                    "Situation Critical. We must show resolve but avoid a first strike. (Recommend: CONTAIN)".to_string()
                                } else if self.state.foreign_paranoia > 0.7 {
                                    "They are scared. Reducing readiness might calm them. (Recommend: STAND DOWN)".to_string()
                                } else {
                                    "We should test their response times. (Recommend: INVESTIGATE)"
                                        .to_string()
                                }
                            }
                            AdvisorRole::Director => {
                                if self.state.secret_weapon_progress > 0.7 {
                                    "The Project is becoming unstable. We need to secure the facility. (Recommend: INVESTIGATE)".to_string()
                                } else if self.state.internal_secrecy < 0.5 {
                                    "Too many leaks. We need to plug the holes. (Recommend: INVESTIGATE)".to_string()
                                } else {
                                    "We can use the confusion to our advantage. (Recommend: LEAK)"
                                        .to_string()
                                }
                            }
                            AdvisorRole::Ambassador => {
                                if self.state.global_tension > 0.6 {
                                    "We need a backchannel. I can arrange a meeting. (Recommend: CONTAIN)".to_string()
                                } else if self.state.domestic_stability < 0.4 {
                                    "The people need to know we are working for peace. (Recommend: LEAK)".to_string()
                                } else {
                                    "Maintain current diplomatic pressure. (Recommend: WAIT)"
                                        .to_string()
                                }
                            }
                        }
                    };

                    feedback.push(format!("\"{}\"", advice));
                } else {
                    feedback.push(format!("ERROR: ADVISOR '{}' NOT FOUND.", target));
                    // Refund if it cost anything (though we deducted already, so let's refund)
                    if self.consult_count > 0 && self.intel_points < self.max_intel_points {
                        // Only refund if we actually paid.
                        // Logic check: We incremented consult_count, so next one will cost.
                        // Let's just refund the point if we paid.
                        // Actually, simpler: if not found, don't count it.
                        self.consult_count -= 1;
                        // But we already deducted intel if consult_count was > 0 BEFORE increment...
                        // Fix: logic above deducted if consult_count > 0.
                        // If we are here, we might have deducted.
                        // It's a bit messy. Let's just say "Input error = no cost".
                        // Re-adding the point is fine.
                        // But wait, the check was `if self.consult_count > 0`.
                        // If this was the first (0), we didn't pay.
                        // If this was second (1), we paid.
                        // So if we paid, we refund.
                        // Determining if we paid: consult_count was incremented. So current is > 1 means previous was > 0.
                        if self.consult_count > 1 {
                            self.intel_points += 1;
                        }
                    }
                }
            }
            Directive::Decrypt(target_id) => {
                turn_ended = false;
                if self.intel_points == 0 {
                    feedback
                        .push("FAILURE: INSUFFICIENT INTEL ASSETS. YOU MUST ACT NOW.".to_string());
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
                            feedback.push(format!(
                                "NOTICE: DOCUMENT {} WAS NOT ENCRYPTED. (Intel Asset Wasted)",
                                target_id
                            ));
                        }
                        found = true;
                        break;
                    }
                }
                if !found {
                    feedback.push(format!("ERROR: DOCUMENT {} NOT FOUND.", target_id));
                    self.intel_points += 1;
                }
            }
            Directive::Analyze(target_id) => {
                turn_ended = false;
                if self.intel_points == 0 {
                    feedback
                        .push("FAILURE: INSUFFICIENT INTEL ASSETS. YOU MUST ACT NOW.".to_string());
                    return (feedback, false);
                }

                self.intel_points -= 1;
                let mut found = false;
                for doc in &self.pending_documents {
                    if doc.id == target_id {
                        let integrity = (doc.reliability * 100.0) as u32;
                        let assessment = if integrity > 80 {
                            "HIGH (VERIFIED)"
                        } else if integrity > 50 {
                            "MODERATE (UNCERTAIN)"
                        } else {
                            "LOW (POSSIBLE DISINFORMATION)"
                        };

                        feedback.push(format!("ANALYSIS COMPLETE: DOCUMENT {}", target_id));
                        feedback.push(format!(
                            "SOURCE RELIABILITY: {}% - {}",
                            integrity, assessment
                        ));
                        found = true;
                        break;
                    }
                }
                if !found {
                    feedback.push(format!("ERROR: DOCUMENT {} NOT FOUND.", target_id));
                    self.intel_points += 1;
                }
            }
            Directive::Escalate => {
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
            }
            Directive::Investigate => {
                self.state.internal_secrecy -= 0.1;
                self.state.secret_weapon_progress += 0.15;
                feedback.push("Internal audit reveals deeper layers of the Project.".to_string());
                if self.rng.random_bool(0.5) {
                    self.state.accidental_escalation_risk -= 0.1;
                    feedback.push("Protocols tightened. We are watching the watchers.".to_string());
                }
            }
            Directive::Contain => {
                if self.state.foreign_paranoia > 0.6 {
                    feedback.push(
                        "Diplomacy FAILED. Enemy interprets silence as preparation for war."
                            .to_string(),
                    );
                    self.state.global_tension += 0.1;
                } else {
                    self.state.global_tension -= 0.15;
                    self.state.domestic_stability -= 0.1;
                    feedback.push(
                        "Tension reduced. Military leadership questions your resolve.".to_string(),
                    );
                }
            }
            Directive::Leak => {
                self.state.internal_secrecy -= 0.25;
                self.state.domestic_stability += 0.2;
                self.state.foreign_paranoia -= 0.05;
                feedback.push("The truth is out. The public riots, but they trust you more than the Generals.".to_string());
            }
            Directive::StandDown => {
                self.state.global_tension -= 0.4;
                self.state.foreign_paranoia -= 0.3;
                self.state.domestic_stability -= 0.35;
                feedback
                    .push("Total withdrawal ordered. We are naked before our enemies.".to_string());
                feedback.push("Rumors of a military tribunal are circulating.".to_string());
            }
        }

        if turn_ended {
            // PASSIVE ESCALATION
            if self.state.global_tension > 0.3 {
                self.state.global_tension += 0.03;
            }
            if self.state.secret_weapon_progress > 0.2 {
                self.state.secret_weapon_progress += 0.02;
            }

            // Random chance for Red Phone if mole isn't found yet but tension is high
            if self.state.global_tension > 0.8 && self.rng.random_bool(0.1) {
                self.state.red_phone_active = true;
            }

            self.state.global_tension = self.state.global_tension.clamp(0.0, 1.0);
            self.state.internal_secrecy = self.state.internal_secrecy.clamp(0.0, 1.0);
            self.state.foreign_paranoia = self.state.foreign_paranoia.clamp(0.0, 1.0);
            self.state.accidental_escalation_risk =
                self.state.accidental_escalation_risk.clamp(0.0, 1.0);
            self.state.domestic_stability = self.state.domestic_stability.clamp(0.0, 1.0);
            self.state.secret_weapon_progress = self.state.secret_weapon_progress.clamp(0.0, 1.0);

            if self.state.accidental_escalation_risk > 0.6 && self.rng.random_bool(0.3) {
                self.state.global_tension += 0.15;
                feedback.push("WARNING: UNAUTHORIZED SILO ACTIVATION DETECTED.".to_string());
            }

            if self.state.secret_weapon_progress > 0.9 && self.rng.random_bool(0.2) {
                feedback.push(
                    " THE BASILISK IS SPEAKING TO THE OPERATORS. THEY ARE WEEPING.".to_string(),
                );
            }
        }

        (feedback, turn_ended)
    }
}
