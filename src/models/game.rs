use serde::Serialize;

use crate::resources::game_resources::{EvidenceCard, EvidenceCardCategories};

#[derive(Serialize)]
pub struct GameInitialState {
    pub target_card_idxs: Vec<i64>,
    pub murdered_user_id: i64,
}

#[derive(Serialize)]
pub struct GameState<'a> {
    pub murdered_user_idx: i64,
    pub evd_card_cats: &'a Vec<EvidenceCardCategories>,
    pub target_card_idxs: Option<Vec<i64>>,
    pub winner_ids: Option<Vec<i64>>,
}
