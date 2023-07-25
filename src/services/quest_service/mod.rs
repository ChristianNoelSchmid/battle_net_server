pub mod data_layer;
pub mod error;

use std::sync::Arc;

use axum::async_trait;
use rand::{seq::IteratorRandom, thread_rng};

use crate::{
    models::{
        game_models::{Stats, CardModel},
        quest_models::{
            QuestEventMonster, RiddleModel, QuestReward, RiddleStatus, QuestModel,
        },
    },
    resources::game_resources::Resources,
};

use self::{error::{Result, QuestServiceError}, data_layer::QuestDataLayer};

use super::auth_service::AuthService;

#[async_trait]
pub trait QuestService : Send + Sync {
    ///
    /// Generates a new quest of the specified `quest_type` for the user with the given `user_id`
    /// 
    async fn generate_quest(&self, user_id: i32, quest_type: i32) -> Result<QuestModel>;
    async fn guess_riddle(&self, user_id: i32, answer: String) -> Result<RiddleStatus>;
    async fn conf_rand_card(&self, user_id: i32) -> Result<Option<CardModel>>;
    async fn reset_users(&self) -> Result<()>;
}

pub struct CoreQuestService {
    auth_service: Arc<dyn AuthService>,
    data_layer: Arc<dyn QuestDataLayer>,
    res: Arc<Resources>,
}

#[async_trait]
impl QuestService for CoreQuestService {
    async fn generate_quest(&self, user_id: i32, quest_type: i32) -> Result<QuestModel> {
        let quest = self.data_layer.create_new_user_quest(user_id, quest_type).await.map_err(|e| e.into())?;

        match quest_type {
            0 => _ = self.generate_monster_quest(quest.id, quest.lvl).await,
            1 => _ = self.generate_riddle_quest(user_id, quest.id, quest.lvl).await,
            _ => { }
        }

        Ok(quest)
    }

    async fn guess_riddle(&self, user_id: i32, answer: String) -> Result<RiddleStatus> {
        // Lowercase answer for string-matching
        let answer = answer.to_lowercase();
        // Get the user's riddle quest index. Throw error if one isn't found
        // (ie. the user is not on a riddle quest)
        let riddle_idx = self.data_layer.get_quest_riddle_idx(user_id).await.map_err(|e| e.into())?
            .ok_or(QuestServiceError::UserNotOnRiddleQuest)?;

        let riddle = &self.res.riddles[riddle_idx as usize];

        // If the user provides any answer in the collection of answers for the riddle,
        // quest is successfully completed
        if riddle.answers.iter().any(|ans| ans.to_lowercase() == answer) {
            // Complete the quest
            self.data_layer.complete_quest(user_id).await.map_err(|e| e.into())?;
            // Get a new confirmed card
            let new_card = self.data_layer.confirm_rand_card(user_id, &self.res.evd_cats_and_cards).await.map_err(|e| e.into())?;

            // Return a successful riddle status, with the quest reward
            return Ok(
                RiddleStatus::Correct(
                    QuestReward {
                        item_idxs: vec![],
                        card: new_card
                    },
                )
            );
        }
        return Ok(RiddleStatus::Incorrect);
    }

    async fn conf_rand_card(&self, user_id: i32) -> Result<Option<CardModel>> {
        self.data_layer.confirm_rand_card(user_id, &self.res.evd_cats_and_cards).await.map_err(|e| e.into())
    }

    async fn reset_users(&self) -> Result<()> {
        self.data_layer.reset_user_stats(&self.res.user_base_stats).await.map_err(|e| e.into())
    }
}

impl CoreQuestService {
    async fn generate_monster_quest(&self, quest_id: i32, quest_level: i32) -> Result<QuestEventMonster> {
        // Choose a new monster to fight the player
        let (monster_idx, monster) = self.res.monsters
            .iter().enumerate().filter(|(_, monster)| monster.level == quest_level)
            .choose(&mut thread_rng()).unwrap();

        self.data_layer.create_quest_monster(quest_id, monster_idx as i32, monster.stats).await.map_err(|e| e.into())?;

        Ok(QuestEventMonster {
            monster_idx: monster_idx as i64,
            stats: Stats::from_base_stats(monster.stats),
        })
    }

    pub async fn generate_riddle_quest(&self, user_id: i32, quest_id: i32, quest_level: i32) -> Result<Option<RiddleModel>> {
        let ans_riddle_idxs = self.data_layer.get_user_answered_riddles(user_id).await.map_err(|e| e.into())?;

        // Choose a new riddle to give the player,
        // that the player has not seen
        let idx_and_riddle = self.res.riddles
            .iter().enumerate()
            .filter(|(idx, riddle)| riddle.level == quest_level && !ans_riddle_idxs.contains(&(*idx as i32)))
            .choose(&mut thread_rng());

        if let Some((idx, riddle)) = idx_and_riddle {
            self.data_layer.create_quest_riddle(quest_id, idx as i32).await.map_err(|e| e.into())?;

            return Ok(Some(RiddleModel {
                idx: idx as i32,
                text: &riddle.text,
            }));
        } else {
            return Ok(None);
        };
    } 

    /*pub fn equip_item(
        db: &Connection,
        item_idx: i64,
        item_slot: i64,
        user: AuthUser,
        res: &State<Resources>,
    ) -> Result<(), &'static str> {
        // If an item is being equipped, and the user
        // isn't just requesting unequipping
        if item_idx != -1 {
            // Ensure that the item being equipped exists
            // in the user's inventory, unequipped
            let mut item_in_inv = query!(
                db,
                r"SELECT * FROM user_items WHERE user_id = ? AND item_idx = ? AND equip_slot = NULL",
                Value::Integer(user.0),
                Value::Integer(item_idx)
            );
            if let Some(_) = item_in_inv.next() {
                // Select the item that's being replaced, if
                // on exists in the slot
                let prev_item_idx: Option<i64> = query!(
                    db,
                    "SELECT item_idx FROM user_items WHERE user_id = ? AND equip_slot = ?",
                    Value::Integer(user.0),
                    Value::Integer(item_slot)
                )
                .map(|row| row.get("item_idx"))
                .next();

                // If there was an item in that slot, unapply all effects
                if let Some(prev_item) = prev_item_idx.and_then(|idx| Some(&res.items[idx as usize])) {
                    if let Some(effects) = prev_item.effects_self.as_ref() {
                        for effect in effects {
                            effect.to_effect().remove_from_user(db, user.0);
                        }
                    }
                }

                // Retrieve the new item and apply the effects
                let new_item = &res.items[item_idx as usize];
                if let Some(effects) = new_item.effects_self.as_ref() {
                    for effect in effects {
                        effect.to_effect().apply_to_user(db, user.0);
                    }
                }
                execute!(
                    db,
                    r"UPDATE user_items SET equip_slot = ?
                    WHERE user_id = ? AND item_idx = ? LIMIT 1",
                    Value::Integer(item_slot),
                    Value::Integer(user.0),
                    Value::Integer(item_idx)
                );
                return Ok(());
            } else {
                return Err("User does not have item");
            }
        // If the slot is only being unequipped, invoke the command
        } else {
            execute!(
                db,
                r"UPDATE user_items SET equip_slot = NULL
                WHERE user_id = ? AND item_idx = ? LIMIT 1",
                Value::Integer(user.0),
                Value::Integer(item_idx)
            );
            return Ok(());
        }
    }*/

    
}