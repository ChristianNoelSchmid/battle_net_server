use chrono::NaiveDateTime;

#[derive(Debug, Default)]
pub struct UserDbModel { 
    pub id: i64, 
    pub email: String, 
    pub pwd_hash: String, 
}

#[derive(Clone, Default)]
pub struct RefrTokenDbModel {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub repl_id: Option<i64>,
    pub revoked_on: Option<NaiveDateTime>
}