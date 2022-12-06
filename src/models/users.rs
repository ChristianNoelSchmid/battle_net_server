use serde::Serialize;
use sqlite::Row;

use super::model::Model;

#[derive(Serialize, Clone)]
pub struct User {
    pub id: i64,
    pub card_id: i64,
    pub user_name: String,
    pub user_img_path: Option<String>,
}
impl Model for User {
    fn from_row(row: Row) -> Self {
        User {
            id: row.get("id"),
            card_id: row.get("card_id"),
            user_name: row.get("user_name"),
            user_img_path: row.try_get("user_img_path").ok(),
        }
    }
}



#[derive(Serialize, Clone)]
pub struct UserState {
    pub confirmed_card_ids: Vec<i64>,
    pub unconfirmed_card_ids: Vec<i64>,
}