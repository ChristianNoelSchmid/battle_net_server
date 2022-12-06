use serde::Serialize;
use sqlite::Row;

use super::{users::User, model::Model};

#[derive(Serialize)]
pub struct GameInitialState {
    pub target_card_ids: Vec<i64>,
    pub murdered_user: User,
}

#[derive(Serialize)]
pub struct GameState {
    pub murdered_user: User,
    pub categories: Vec<(i64, String)>,
    pub cards: Vec<EvidenceCard>,
    pub target_cards: Option<Vec<EvidenceCard>>,
    pub winners: Option<Vec<User>>,
}

#[derive(Serialize, Clone)]
pub struct EvidenceCard {
    pub card_id: i64,
    pub item_name: String,
    pub item_img_path: Option<String>,
}

impl Model for EvidenceCard {
    fn from_row(row: Row) -> Self {
        EvidenceCard {
            card_id: row.get("id"),
            item_name: row.get("item_name"),
            item_img_path: row.get("item_img_path"),
        }
    }
}