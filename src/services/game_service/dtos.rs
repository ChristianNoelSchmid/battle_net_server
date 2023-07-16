use serde::Serialize;

use crate::resources::game_resources::EvidenceCardCategories;

use super::data_layer::entities::CardModel;

#[derive(Serialize)]
pub struct GameInitialStateDto {
    pub target_cards: Vec<CardModel>,
    pub murdered_user_id: i64,
}

#[derive(Serialize)]
pub struct GameStateDto {
    pub murdered_user_idx: i64,
    pub evd_cats_and_cards: Vec<EvidenceCardCategories>,
    pub target_card_idxs: Option<Vec<CardModel>>,
    pub winner_ids: Option<Vec<i64>>,
}

