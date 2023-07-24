
pub mod entities;

use std::collections::HashSet;

use axum::async_trait;
use chrono::{Datelike, Utc};
use rand::{seq::IteratorRandom, rngs::StdRng, SeedableRng};

use crate::{data_layer_error::Result, models::quest_models::QuestModel, resources::game_resources::{BaseStats, EvidenceCardCategories}, prisma::{PrismaClient, quest, user, stats, quest_riddle, user_card}, services::game_service::data_layer::entities::CardEntity};

#[async_trait]
pub trait QuestDataLayer : Send + Sync {
    ///
    /// Retrieves the current-day quest (if one exists) of the user specified by `user_id`
    /// Quests are active if they are both not marked as `completed`, and are from the current day.
    /// 
    async fn get_active_user_quest(&self, user_id: i32) -> Result<Option<QuestModel>>;
    ///
    /// Creates a battle quest for the specified user, by `user_id`.
    /// If there is already an existing quest for the day and it's not completed,
    /// this increments the quest level and activates it
    /// 
    async fn create_new_user_quest(&self, user_id: i32, quest_type: i32) -> Result<QuestModel>;
    ///
    /// Completes the quest of the user with the given `user_id`
    /// 
    async fn complete_quest(&self, user_id: i32) -> Result<()>;
    ///
    /// Creates a new monster with the specified stats, and assigns to the given quest
    /// 
    async fn create_quest_monster(&self, quest_id: i32, monster_idx: i32, stats: BaseStats) -> Result<()>;
    ///
    /// Retrieves the indices of all riddles the user has answered
    /// 
    async fn get_user_answered_riddles(&self, user_id: i32) -> Result<Vec<i32>>;
    ///
    /// Creates a new riddle with the specified index, and assigns to the given quest
    /// 
    async fn create_quest_riddle(&self, quest_id: i32, riddle_idx: i32) -> Result<()>;
    ///
    /// Retrieves the index of the current user's quest's riddle
    /// 
    async fn get_quest_riddle_idx(&self, user_id: i32) -> Result<Option<i32>>;
    /// 
    /// Confirms a new, random evidence card, if any exist that has yet to be confirmed
    /// in the user's collection
    /// 
    async fn confirm_rand_card<'a>(&self, user_id: i32, all_cards: &'a [EvidenceCardCategories]) -> Result<Option<CardEntity>>;
    ///
    /// Resets the stats for all users
    /// 
    async fn reset_user_stats<'a>(&self, base_stats: &'a BaseStats) -> Result<()>;
}

pub struct DbQuestDataLayer {
    db: PrismaClient
}

#[async_trait]
impl QuestDataLayer for DbQuestDataLayer {
    async fn get_active_user_quest(&self, user_id: i32) -> Result<Option<QuestModel>> {
        // Get the most recent quest for the user (if one exists)
        let quest = self.db.quest().find_first(vec![quest::user_id::equals(user_id), quest::completed::equals(false)])
            .exec().await.map_err(|e| Box::new(e))?;

        if let Some(quest) = quest {
            // Check if the quest is from today - if not, return None. Otherwise, return the quest
            let today = Utc::now().naive_utc();
            if quest.created_on.num_days_from_ce() == today.num_days_from_ce() {
                return Ok(Some(QuestModel { 
                    id: quest.id, created_on: quest.created_on, user_id: quest.user_id, 
                    lvl: quest.lvl, quest_type: quest.quest_type, completed: quest.completed
                }));
            }
        }

        Ok(None)
    }
    async fn create_new_user_quest(&self, user_id: i32, quest_type: i32) -> Result<QuestModel> {
        // Determine if the user has a daily quest already
        let daily_quest = self.get_active_user_quest(user_id).await?;
        let mut lvl = 1;

        // If there is a daily quest, mark it as completed, and set lvl to incremented value
        if let Some(daily_quest) = daily_quest {
            self.db.quest().update(quest::UniqueWhereParam::IdEquals(daily_quest.id), vec![quest::completed::set(true)])
                .exec().await.map_err(|e| Box::new(e))?;

            lvl = daily_quest.lvl + 1;
        } 

        // Create the new quest
        self.db.quest().create(quest_type, user::id::equals(user_id), vec![quest::lvl::set(lvl)])
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(self.get_active_user_quest(user_id).await?.unwrap())
    }

    async fn create_quest_monster(&self, quest_id: i32, monster_idx: i32, stats: BaseStats) -> Result<()> {
        let stats = self.db.stats().create(stats.health, stats.magicka, stats.armor, stats.wisdom, stats.reflex, false, vec![])
            .exec().await.map_err(|e| Box::new(e))?;
        self.db.quest_monster().create(quest::id::equals(quest_id), stats::id::equals(stats.id), vec![])
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(())
    }

    async fn get_user_answered_riddles(&self, user_id: i32) -> Result<Vec<i32>> {
        // Get all user completed quests that are riddle quests
        let quest_ids = self.db.quest().find_many(vec![
            quest::user_id::equals(user_id), 
            quest::completed::equals(true),
            quest::quest_type::equals(1)
        ])
            .exec().await.map_err(|e| Box::new(e))?
            .iter().map(|q| q.id).collect();

        // Get all riddle idxs using the riddle quest ids
        let riddle_idxs = self.db.quest_riddle().find_many(
            vec![quest_riddle::quest_id::in_vec(quest_ids)]
        )
            .exec().await.map_err(|e| Box::new(e))?
            .iter().map(|r| r.riddle_idx).collect();

        Ok(riddle_idxs)
    }

    async fn create_quest_riddle(&self, quest_id: i32, riddle_idx: i32) -> Result<()> {
        self.db.quest_riddle().create(riddle_idx, quest::id::equals(quest_id), vec![])
            .exec().await.map_err(|e| Box::new(e))?;
        Ok(())
    }

    async fn get_quest_riddle_idx(&self, user_id: i32) -> Result<Option<i32>> {
        let quest = self.get_active_user_quest(user_id).await?;
        if let Some(quest) = quest {
            if quest.quest_type == 1 {
                let riddle_idx = self.db.quest_riddle().find_first(vec![quest_riddle::quest_id::equals(quest.id)])
                    .exec().await.map_err(|e| Box::new(e))?.unwrap().riddle_idx;

                return Ok(Some(riddle_idx));
            }
        }
        Ok(None)
    }

    async fn complete_quest(&self, user_id: i32) -> Result<()> {
        self.db.quest().update_many(
            vec![quest::user_id::equals(user_id), quest::completed::equals(false)],
            vec![quest::completed::set(true)]
        )
            .exec().await.map_err(|e| Box::new(e))?;
        Ok(())
    }

    async fn confirm_rand_card<'a>(&self, user_id: i32, cards: &'a [EvidenceCardCategories]) -> Result<Option<CardEntity>> {
        let mut rng = StdRng::from_entropy();

        // Create an iterator of all permutations of cat idx to card idx
        let card_cat_pairs = cards.iter().enumerate()
            .flat_map(|(cat_idx, cat)| (0..cat.cards.len()).map(move |card_idx| (cat_idx as i32, card_idx as i32)));

        // Get all confirmed user cards, convert into 2-ples of cat and card idxs
        // and collect into a HashSet
        let conf_cat_card_idxs = self.db.user_card()
            .find_many(vec![user_card::user_id::equals(user_id), user_card::confirmed::equals(true)])
            .exec().await.map_err(|e| Box::new(e))?
            .iter().map(|conf_card| (conf_card.cat_idx, conf_card.card_idx))
            .collect::<HashSet<(i32, i32)>>();
        
        // Choose a random cat and card idx that isn't in the confirmed collection
        let choice = card_cat_pairs.filter(|pair| !conf_cat_card_idxs.contains(pair)).choose(&mut rng);
        Ok(choice.and_then(|choice| Some(CardEntity { cat_idx: choice.0, card_idx: choice.1 })))
    }

    async fn reset_user_stats<'a>(&self, base_stats: &'a BaseStats) -> Result<()> {
        // Get all user stats ids
        let stats_ids = self.db.user().find_many(vec![]).with(user::state::fetch())
            .exec().await.map_err(|e| Box::new(e))?
            .iter().map(|user| user.state.as_ref().unwrap().as_ref().unwrap().stats_id).collect();

        // Update all found stats
        self.db.stats().update_many(vec![stats::id::in_vec(stats_ids)], vec![
            stats::health::set(base_stats.health),
            stats::magicka::set(base_stats.magicka),
            stats::armor::set(base_stats.armor),
            stats::wisdom::set(base_stats.wisdom),
            stats::reflex::set(base_stats.reflex),
            stats::missing_next_turn::set(false),
        ])
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(())
    }
}