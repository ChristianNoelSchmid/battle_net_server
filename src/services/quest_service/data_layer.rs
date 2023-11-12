
use std::{collections::HashSet, sync::Arc};

use axum::async_trait;
use chrono::{Datelike, Utc};
use derive_more::Constructor;
use prisma_client_rust::Direction;
use rand::{seq::IteratorRandom, rngs::StdRng, SeedableRng};

use crate::{data_layer_error::Result, resources::game_resources::{BaseStats, EvidenceCardCategories}, prisma::{PrismaClient, quest, user, stats, quest_riddle, user_card, quest_monster}, services::game_service::models::{CardModel, Stats}};

use super::entities::{QuestMonsterEntity, QuestStateEntity};

#[async_trait]
pub trait QuestDataLayer : Send + Sync {
    ///
    /// Retrieves the current-day quest (if one exists) of the user specified by `user_id`
    /// Quests are active if they are both not marked as `completed`, and are from the current day.
    /// 
    async fn get_active_user_quest(&self, user_id: i32) -> Result<Option<QuestStateEntity>>;
    ///
    /// Retrieves the player's current level. Players level up when they complete monster quests.
    /// 
    async fn get_pl_lvl(&self, user_id: i32) -> Result<i32>;
    ///
    /// Retrieves whether the player has completed a riddle quest today
    /// 
    async fn pl_answered_riddle(&self, user_id: i32) -> Result<bool>;
    ///
    /// Creates a battle quest for the specified user, by `user_id`.
    /// If there is already an existing quest for the day and it's completed,
    /// this increments the quest level and activates it. If it's not completed
    /// returns None
    /// 
    async fn create_new_user_quest(&self, user_id: i32, quest_type: i32) -> Result<Option<QuestStateEntity>>;
    ///
    /// Completes the quest of the user with the given `user_id`
    /// 
    async fn complete_quest(&self, user_id: i32) -> Result<()>;
    ///
    /// Exhausts the player, preventing them from performing any more battle quests that day
    /// 
    async fn exhaust_pl(&self, user_id: i32) -> Result<()>;
    ///
    /// Returns whether the player is exhausted today
    /// 
    async fn pl_is_exhausted(&self, user_id: i32) -> Result<bool>;
    ///
    /// Creates a new monster with the specified stats, and assigns to the given quest
    /// 
    async fn create_quest_monster(&self, quest_id: i32, monster_idx: i32, stats: BaseStats) -> Result<()>;
    ///
    /// Retrieves the indices of all riddles the user has answered
    /// 
    async fn get_user_answered_riddle(&self, user_id: i32) -> Result<Vec<i32>>;
    ///
    /// Creates a new riddle with the specified index, and assigns to the given quest
    /// 
    async fn create_quest_riddle(&self, quest_id: i32, riddle_idx: i32) -> Result<()>;
    ///
    /// Retrieves the index of the current user's quest's riddle
    /// 
    async fn get_quest_riddle_idx(&self, user_id: i32) -> Result<Option<i32>>;
    /// 
    /// Retrieves a new, random evidence card, if any exist that has yet to be confirmed
    /// in the user's collection
    /// 
    async fn get_rand_unconfirmed_card<'a>(&self, user_id: i32, all_cards: &'a [EvidenceCardCategories]) -> Result<Option<CardModel>>;
    ///
    /// Deletes the quest with the given id
    /// 
    async fn delete_quest(&self, quest_id: i32) -> Result<()>;
}

#[derive(Constructor)]
pub struct DbQuestDataLayer {
    db: Arc<PrismaClient>
}

#[async_trait]
impl QuestDataLayer for DbQuestDataLayer {
    async fn get_active_user_quest(&self, user_id: i32) -> Result<Option<QuestStateEntity>> {
        // Get the most recent, incomplete quest for the user (if one exists)
        let quest = self.db.quest().find_first(vec![quest::user_id::equals(user_id), quest::completed::equals(false)])
            .order_by(quest::OrderByParam::CreatedOn(Direction::Desc))
            .exec().await.map_err(|e| Box::new(e))?;

        if let Some(quest) = quest {
            // Check if the quest is from today - if not, return None. Otherwise, return the quest
            let today = Utc::now().naive_utc();
            if quest.created_on.num_days_from_ce() == today.num_days_from_ce() {

                // Get the monster state for the quest if it's a monster quest
                let monster_state = self.db.quest_monster().find_first(vec![quest_monster::quest_id::equals(quest.id)])
                    .with(quest_monster::stats::fetch())
                    .exec().await.map_err(|e| Box::new(e))?
                    .and_then(|ms| {
                        let stats = ms.stats.unwrap();
                        Some(QuestMonsterEntity {
                            monster_idx: ms.monster_idx,
                            stats: Stats::new(stats.health, stats.power, stats.armor, stats.missing_next_turn)
                        })
                    });

                // Get the riddle idx for the quest if it's a riddle quest
                let riddle_idx = self.db.quest_riddle().find_first(vec![quest_riddle::quest_id::equals(quest.id)])
                    .exec().await.map_err(|e| Box::new(e))?
                    .and_then(|r| Some(r.riddle_idx));

                return Ok(Some(QuestStateEntity { 
                    id: quest.id, quest_type: quest.quest_type,
                    monster_state, riddle_idx,
                    completed: quest.completed
                }));
            }
        }

        Ok(None)
    }
    async fn create_new_user_quest(&self, user_id: i32, quest_type: i32) -> Result<Option<QuestStateEntity>> {
        // Determine if the user has a daily quest already
        let daily_quest = self.get_active_user_quest(user_id).await?;

        // If there is a daily quest and it's completed, create new quest and set lvl to incremented value
        if let Some(daily_quest) = daily_quest {
            // Return None if the quest has not been completed yet
            if !daily_quest.completed {
                return Ok(None);
            }
        } 

        // Create the new quest
        self.db.quest().create(quest_type, user::id::equals(user_id), vec![])
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(Some(self.get_active_user_quest(user_id).await?.unwrap()))
    }

    async fn get_pl_lvl(&self, user_id: i32) -> Result<i32> {
        Ok(self.db.user().find_unique(user::UniqueWhereParam::IdEquals(user_id))
            .exec().await.map_err(|e| Box::new(e))?.unwrap().lvl)
    }

    async fn pl_answered_riddle(&self, user_id: i32) -> Result<bool> {
        Ok(self.db.user().find_unique(user::UniqueWhereParam::IdEquals(user_id))
            .exec().await.map_err(|e| Box::new(e))?.unwrap().riddle_quest_completed)
    }

    async fn create_quest_monster(&self, quest_id: i32, monster_idx: i32, stats: BaseStats) -> Result<()> {
        let stats = self.db.stats().create(stats.health, stats.armor, false, vec![])
            .exec().await.map_err(|e| Box::new(e))?;
        self.db.quest_monster().create(monster_idx, quest::id::equals(quest_id), stats::id::equals(stats.id), vec![])
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(())
    }

    async fn get_user_answered_riddle(&self, user_id: i32) -> Result<Vec<i32>> {
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
        // Get the quest's id
        let quest = self.db.quest().find_first(vec![quest::user_id::equals(user_id), quest::completed::equals(false)])
            .exec().await.map_err(|e| Box::new(e))?.unwrap();

        // Update the quest as completed
        self.db.quest().update(
            quest::UniqueWhereParam::IdEquals(quest.id),
            vec![quest::completed::set(true)]
        )
            .exec().await.map_err(|e| Box::new(e))?;

        if quest.quest_type == 0 {
            // If it was a monster battle, set the user's lvl to 2
            self.db.user().update(user::UniqueWhereParam::IdEquals(user_id), vec![user::lvl::set(2)])
                .exec().await.map_err(|e| Box::new(e))?;
        } else {
            // If it was a riddle quest, update the user as completed a riddle today
            self.db.user().update(user::UniqueWhereParam::IdEquals(user_id), vec![user::riddle_quest_completed::set(true)])
                .exec().await.map_err(|e| Box::new(e))?;
        }


        Ok(())
    }

    async fn exhaust_pl(&self, user_id: i32) -> Result<()> {
        self.db.user().update(user::UniqueWhereParam::IdEquals(user_id), vec![user::exhausted::set(true)])
            .exec().await.map_err(|e| Box::new(e))?;
        Ok(())
    }

    async fn pl_is_exhausted(&self, user_id: i32) -> Result<bool> {
        Ok(self.db.user().find_unique(user::UniqueWhereParam::IdEquals(user_id))
            .exec().await.map_err(|e| Box::new(e))?.unwrap().exhausted)
    }

    async fn get_rand_unconfirmed_card<'a>(&self, user_id: i32, cards: &'a [EvidenceCardCategories]) -> Result<Option<CardModel>> {
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
        Ok(choice.and_then(|choice| Some(CardModel { cat_idx: choice.0, card_idx: choice.1 })))
    } 

    async fn delete_quest(&self, quest_id: i32) -> Result<()> {
        self.db.quest().delete(quest::UniqueWhereParam::IdEquals(quest_id))
            .exec().await.map_err(|e| Box::new(e))?;
        Ok(())
    }
}