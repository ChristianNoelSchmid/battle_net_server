use chrono::{FixedOffset, DateTime};

#[derive(Debug, Default)]
pub struct UserModel { 
    pub id: i32, 
    pub email: String, 
    pub pwd_hash: String, 
}

#[derive(Clone, Default)]
pub struct RefrTokenModel {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub repl_id: Option<i32>,
    pub revoked_on: Option<DateTime<FixedOffset>>
}