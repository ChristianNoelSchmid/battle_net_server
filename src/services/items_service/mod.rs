mod error;
pub mod data_layer;
use std::sync::Arc;

use crate::resources::game_resources::Resources;

use self::{error::Result, data_layer::DataLayer};

use axum::async_trait;

use super::{game_service::models::Stats};


#[async_trait]
pub trait ItemsService : Send + Sync {
    async fn equip_item(&self, user_id: i32, item_idx: i32, equip: bool) -> Result<Stats>;
}

pub struct CoreItemsService {
    item_data_layer: Arc<dyn self::data_layer::DataLayer>,
    effects_data_layer: Arc<dyn effects_service::data_layer::DataLayer>,
    res: Arc<Resources>
}

#[async_trait]
impl ItemsService for CoreItemsService {
    async fn equip_item(&self, user_id: i32, item_idx: i32, equip: bool) -> Result<Stats> {
        let stats_id = self.item_data_layer.get_user_stats_id(user_id).await.map_err(|e| e.into())?;
        let item = &self.res.items[item_idx as usize];

        for effect_type in item.effects_self.as_ref().unwrap() {
            effect_type.to_effect().apply_to_stats(&self.effects_data_layer, stats_id).await.map_err(|e| e.into())?;
        }

        self.item_data_layer.get_user_stats(user_id).await.map_err(|e| e.into())
    }
}