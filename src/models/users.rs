use serde::Serialize;

#[derive(Serialize)]
pub struct User {
    pub id: i64,
    pub card_idx: i64,
    pub username: String,
    pub passwd: String,
}

#[derive(Serialize, Clone)]
pub struct UserState {
    pub confirmed_card_ids: Vec<i64>,
    pub unconfirmed_card_ids: Vec<i64>,
}
