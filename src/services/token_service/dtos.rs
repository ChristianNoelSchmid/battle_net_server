use tower_cookies::Cookie;

#[derive(Debug, PartialEq)]
pub struct TokensDto {
    pub access_token: String,
    pub refr_token: Cookie<'static>,
}