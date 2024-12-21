pub mod error;
pub mod data_layer;
pub mod settings;

use crate::resources::game_resources::Resources;

use self::error::Result;

use std::sync::Arc;
use self::data_layer::DataLayer;

use chrono::Utc;
use log::info;
use settings::Settings;
use tokio_cron_scheduler::Job;

/// 
/// Refreshes users stats data daily, at midnight
/// 
pub fn create_refresh_job(data_layer: Arc<dyn DataLayer>, res: Arc<Resources>, settings: Settings) -> Result<Job> {
    Ok(
        Job::new_async(
            settings.refresh_rate_cron, 
            move |_uuid, _l| { 
                let dl = data_layer.clone();
                let rs = res.clone();
                Box::pin(async move {
                    let ids = &dl.reset_user_stats(&rs.user_base_stats).await.unwrap();
                    info!("Refreshed user stats @{}. Ids: {:?}", Utc::now(), ids);
                }) 
            }
        ).unwrap()
    )
}