use serde::Serialize;

use super::{game::Stats, quests::QuestEvent};

#[derive(Serialize)]
pub struct User {
    pub id: i64,
    pub card_idx: i64,
    pub username: String,
    pub passwd: String,
}

#[derive(Serialize)]
pub struct UserState<'a> {
    pub evidence_cards: Vec<UserEvidenceCard>,
    pub stats: Stats,
    pub quest_event: Option<QuestEvent<'a>>,
    pub item_idxs_and_slots: Vec<(i64, Option<i64>)>,
}

#[derive(Serialize, Clone)]
pub struct UserEvidenceCard {
    pub cat_idx: i64,
    pub card_idx: i64,
    pub confirmed: bool,
}
