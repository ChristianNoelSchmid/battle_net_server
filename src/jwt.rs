use dotenvy::dotenv;
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha256;
use std::{collections::BTreeMap, env};

pub fn generate_access_token(user_id: i64) -> String {
    dotenv().ok();
    let key: Hmac<Sha256> = Hmac::new_from_slice(
        env::var("JWT_SECRET")
            .expect("Could not find JWT_SECRET in env variables")
            .as_bytes(),
    )
    .expect("Could not sign JWT token. Please see admin");

    let mut claims = BTreeMap::new();
    claims.insert("user_id", user_id);
    let token_str = claims.sign_with_key(&key);

    token_str.expect("Could not sign JWT token. Please see admin.")
}

pub fn verify_token(access_token: &str) -> Option<i64> {
    dotenv().ok();
    let key: Hmac<Sha256> = Hmac::new_from_slice(
        env::var("JWT_SECRET")
            .expect("Could not find JWT_SECRET in env variables")
            .as_bytes(),
    )
    .expect("Could not sign JWT token. Please see admin");

    let claims: BTreeMap<String, i64> = access_token
        .verify_with_key(&key)
        .expect("Could not verify access token. Please see admin.");

    claims.get("user_id").and_then(|id| Some(*id))
}
