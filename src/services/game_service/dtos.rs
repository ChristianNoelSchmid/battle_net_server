use serde::Serialize;

use crate::resources::game_resources::EvidenceCardCategories;

use super::data_layer::entities::CardEntity;

///
/// Initial state of a game, directly after setup
/// 
#[derive(Serialize)]
pub struct GameInitialStateDto {
    pub target_cards: Vec<CardEntity>,
    pub murdered_user_id: i64,
}

///
/// Current state of a game, including
/// the specific user's evidence cards
/// 
#[derive(Serialize)]
pub struct GameStateDto {
    pub murdered_user_idx: i64,
    pub evd_cats_and_cards: Vec<EvidenceCardCategories>,
    pub target_card_idxs: Option<Vec<CardEntity>>,
    pub winner_ids: Option<Vec<i64>>,
}

