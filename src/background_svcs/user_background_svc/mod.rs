pub mod error;
pub mod data_layer;

use crate::resources::game_resources::Resources;

use self::error::Result;

use std::{sync::Arc, time::Duration};
use self::data_layer::DataLayer;

use chrono::{Utc, Datelike};
use log::info;

/// 
/// Refreshes users stats data daily, at midnight
/// 
pub async fn refresh_daily_async(data_layer: Arc<dyn DataLayer>, res: Arc<Resources>) -> Result<()> {
    loop {
        let now = Utc::now().fixed_offset();
        let last_update = data_layer.get_last_user_refr().await.map_err(|e| e.into())?;

        if let Some(last_update) = last_update {
            if now.naive_utc().num_days_from_ce() > last_update.naive_utc().num_days_from_ce() {
                let ids = data_layer.reset_user_stats(&res.user_base_stats).await.map_err(|e| e.into())?;
                info!("Refreshed user stats @{}. Ids: {:?}", now, ids);
            } 
        }
        
        // Wait for 30 seconds before checking again
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}