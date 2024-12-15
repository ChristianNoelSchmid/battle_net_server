pub mod error;
pub mod models;
pub mod settings;

use std::collections::BTreeMap;

use base64::{engine::general_purpose, Engine};
use chrono::{Duration, Utc, DateTime};
use derive_more::Constructor;
use dotenv_codegen::dotenv;
use dotenvy::dotenv;
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use lazy_static::lazy_static;
use rand::{rngs::OsRng, Rng};
use sha2::Sha256;


use self::{settings::TokenSettings, error::{Result, TokenError}, models::AuthTokensModel};

const REFRESH_TOKEN_LENGTH: usize = 128;

lazy_static! {
    static ref JWT_SECRET: String = {
        dotenv().ok().expect(".env file must be provided");
        dotenv!("JWT_SECRET").to_string()
    };
}

pub trait TokenService: Send + Sync {
    ///
    /// Using the given `info`, generates a JWT and a series of series of
    /// random bytes representing a refresh token.
    ///
    fn generate_auth_tokens(&self, user_id: i64) -> AuthTokensModel;

    ///
    /// Verifies a JWT `token`, and returns the corresponding user ID from the its content section with successful verification.
    /// Returns the associated user ID, or `Error` in the event of unsuccessful verification
    ///
    fn verify_access_token(&self, access_token: &str) -> Result<i64>;
}

#[derive(Clone, Constructor)]
pub struct CoreTokenService {
    settings: TokenSettings
}

impl TokenService for CoreTokenService {
    fn generate_auth_tokens(&self, user_id: i64) -> AuthTokensModel {
        let key: Hmac<Sha256> = Hmac::new_from_slice(JWT_SECRET.as_bytes())
            .expect("error converting SECRET into Hmac<Sha26>");

        let mut claims = BTreeMap::new();
        claims.insert("user_id", user_id.to_string());

        let expires = (chrono::Utc::now() + Duration::seconds(self.settings.jwt_lifetime_s)).to_rfc3339();
        claims.insert("expires", expires);

        let access_token = claims.sign_with_key(&key).expect("error signing jwt key");
        let refresh_token = generate_random_bytes();

        AuthTokensModel {
            access_token,
            refresh_token,
        }
    }

    fn verify_access_token(&self, token: &str) -> Result<i64> {
        // Convert the JWT_SECRET env variable into a Hmac hasher
        let key: Hmac<Sha256> = Hmac::new_from_slice(JWT_SECRET.as_bytes())
            .expect("error converting SECRET into Hmac<Sha256>");

        // Verify the JWT using the hash key
        let claims: std::result::Result<BTreeMap<String, String>, jwt::Error> = token.verify_with_key(&key);

        // If successful verification...
        return match claims {
            Ok(mut claims) => {
                // Check the expires parameter, and return error if the token is stale
                if Utc::now() > DateTime::parse_from_rfc3339(&claims["expires"]).unwrap() {
                    return Err(TokenError::TokenStale);

                // Otherwise, return Ok with the user ID from the token
                } else {
                    return Ok(claims.remove("user_id").unwrap().parse::<i64>().unwrap());
                }
            },
            Err(e) => Err(TokenError::JwtError(e))
        };
    }
}

///
/// Generates a series of random, OS bytes, with a length equal to `REFRESH_TOKEN_LENGTH`
///
fn generate_random_bytes() -> String {
    let mut rng = OsRng::default();
    let mut bytes = [0u8; REFRESH_TOKEN_LENGTH];
    rng.fill(&mut bytes);

    general_purpose::STANDARD_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    struct TokenContents {
        user_id: String,
        expires: String,
    }

    #[test]
    fn test_token_gen_and_verify() {

        let svc = CoreTokenService { settings: TokenSettings { jwt_lifetime_s: 5, refr_token_lifetime_s: 5 }};
        let user_id = 10;

        let tokens = svc.generate_auth_tokens(user_id);

        let token_user_id = svc.verify_access_token(&tokens.access_token);
        assert!(token_user_id.is_ok());

        let token_user_id = token_user_id.unwrap();
        assert_eq!(token_user_id, user_id);
    }

    #[test]
    fn test_improper_token() {
        let svc = CoreTokenService { settings: TokenSettings { jwt_lifetime_s: 5, refr_token_lifetime_s: 5 }};
        let user_id = 10;
        let tokens = svc.generate_auth_tokens(user_id);

        // Grab the content of the JWT, deserialize it, and update the role to "Admin"
        // to attempt to hack the role requirements
        let str = tokens.access_token.split('.').skip(1).next().unwrap();
        let str = general_purpose::STANDARD_NO_PAD.decode(str).unwrap();
        let mut contents: TokenContents =
            serde_json::from_str(&String::from_utf8_lossy(&str).to_string()).unwrap();

        contents.user_id = "10".to_string();

        // Build the new token with the new role, but with the same
        // header and key
        let new_token = format!(
            "{}.{}.{}",
            tokens.access_token.split('.').next().unwrap(),
            general_purpose::STANDARD_NO_PAD.encode(serde_json::to_string(&contents).unwrap()),
            tokens.access_token.split('.').skip(2).next().unwrap()
        );

        // Assert that an error is thrown when the token is attempted to
        // be verified
        let verified_info = svc.verify_access_token(&new_token);
        assert!(verified_info.is_err());
    }
}

