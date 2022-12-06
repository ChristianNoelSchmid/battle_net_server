use serde::Serialize;

use super::game::EvidenceCard;

#[derive(Serialize, Clone)]
pub struct Riddle {
    pub id: i64,
    pub text: String,
}

#[derive(Serialize)]
pub enum RiddleProgress {
    Correct((Option<Riddle>, Option<EvidenceCard>)),
    Incorrect,
}

#[derive(Serialize)]
pub enum QuestEvent {
    Monster(i64),
    Riddle(i64),
}