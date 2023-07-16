#[derive(Debug, PartialEq)]
pub struct CreateRefrTokenDto {
    pub user_id: i64,
    pub token: String
}