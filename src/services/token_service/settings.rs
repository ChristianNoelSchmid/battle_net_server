use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct TokenSettings {
    pub jwt_lifetime_s: i64,
    pub refr_token_lifetime_s: i64,
}