use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Settings {
    pub refresh_rate_cron: String
}