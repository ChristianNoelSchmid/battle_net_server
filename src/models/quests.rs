use serde::{Deserialize, Serialize};

use crate::{
    effects::Effect,
    resources::game_resources::{EvidenceCard, Riddle},
};

pub struct Item<'a> {
    tag: String,
    name: String,
    flavor_text: String,
    expend: bool,
    effects: Vec<&'a dyn Effect>,
}

#[derive(Serialize)]
pub enum QuestEvent {
    Monster(i64),
    Riddle(i64),
}

#[derive(Serialize)]
pub enum GuessRiddleResult {
    Correct,
    Incorrect,
}
