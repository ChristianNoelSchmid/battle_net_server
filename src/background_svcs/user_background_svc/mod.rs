pub mod error;
pub mod data_layer;

use crate::resources::game_resources::Resources;

use self::error::Result;

use std::sync::Arc;
use self::data_layer::DataLayer;

use chrono::{Datelike, Utc, FixedOffset, TimeZone};
use log::info;

const HOUR_SECS: i32 = 3600;

/// 
/// Refreshes users stats data daily, at midnight
/// 
pub async fn refresh_daily_async(data_layer: Arc<dyn DataLayer>, res: Arc<Resources>) -> Result<()> {
    loop {
        // Targeting UTC - 9
        let timezone = FixedOffset::west_opt(9 * HOUR_SECS).unwrap();
        let now = timezone.from_utc_datetime(&Utc::now().naive_utc());

        let last_update = data_layer.get_last_user_refr().await.map_err(|e| e.into())?;

        if let Some(last_update) = last_update {
            let last_update = timezone.from_utc_datetime(&last_update.naive_utc());
            if now.num_days_from_ce() > last_update.num_days_from_ce() {
                let ids = data_layer.reset_user_stats(&res.user_base_stats).await.map_err(|e| e.into())?;
                info!("Refreshed user stats @{}. Ids: {:?}", now, ids);
            } 
        }
        
        // Wait for 30 seconds before checking again
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}