use lazy_static::lazy_static;

use regex::Regex;
use rocket::{
    http::{Cookie, Status},
    request::{FromRequest, Outcome},
};
use sqlite::Value;

use crate::{
    jwt::{generate_access_token, verify_token},
    query,
    sqlite::db,
};

#[derive(Debug, Clone, Copy)]
pub struct AuthUser(pub i64);

#[derive(Debug)]
pub struct AuthUserError;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = AuthUserError;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        // Attempt to log in by reading the username and password
        // from the request query parameters
        if let Some(Ok(username)) = req.query_value::<String>("username") {
            if let Some(Ok(test_passwd)) = req.query_value::<String>("passwd") {
                if let Some(id) = credentials_match(&username, &test_passwd) {
                    let token = generate_access_token(id);
                    let cookie = Cookie::build("accessToken", token)
                        .path("/")
                        .secure(!cfg!(debug_assertions))
                        .http_only(!cfg!(debug_assertions))
                        .permanent()
                        .finish();

                    req.cookies().add(cookie);
                    return Outcome::Success(AuthUser(id));
                }
            }
        }

        // Next, try to get the credentials from the request cookie (if it exists)
        if let Some(secret) = req.cookies().get("accessToken") {
            if let Some(id) = verify_token(secret.value()) {
                return Outcome::Success(AuthUser(id));
            }
        }

        Outcome::Failure((Status::Unauthorized, AuthUserError))
    }
}

fn credentials_match(username: &str, test_passwd: &str) -> Option<i64> {
    let db = db();

    // Try to get the user corresponding to the params
    let mut rows = query!(
        db,
        "SELECT id, passwd FROM users WHERE username = ?",
        Value::String(username.to_string())
    );

    if let Some(user) = rows.next() {
        let id = user.get("id");
        let passwd: String = user.get("passwd");

        if passwd == test_passwd {
            return Some(id);
        }
    }
    None
}
