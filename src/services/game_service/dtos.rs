use serde::Serialize;

use crate::resources::game_resources::EvidenceCardCategories;

use super::data_layer::entities::CardEntity;



///
/// Current state of a game, including
/// the specific user's evidence cards
/// 
#[derive(Serialize)]
pub struct GameStateDto {
    pub murdered_user_idx: i32,
    pub evd_cats_and_cards: Vec<EvidenceCardCategories>,
    pub target_card_idxs: Option<Vec<CardEntity>>,
    pub winner_ids: Option<Vec<i64>>,
}

