
use std::collections::HashSet;
use axum::async_trait;
use chrono::{Datelike, Utc};
use derive_more::Constructor;
use rand::{seq::IteratorRandom, rngs::StdRng, SeedableRng};
use sqlx::SqlitePool;

use crate::{data_layer_error::Result, resources::game_resources::{BaseStats, EvidenceCardCategories}, services::game_service::models::{CardModel, Stats}};

use super::entities::{QuestMonsterEntity, QuestStateEntity};

#[async_trait]
pub trait QuestDataLayer : Send + Sync {
    async fn pl_has_won_game(&self, user_id: i64) -> Result<bool>;
    ///
    /// Retrieves the current-day quest (if one exists) of the user specified by `user_id`
    /// Quests are active if they are both not marked as `completed`, and are from the current day.
    /// 
    async fn get_active_user_quest(&self, user_id: i64) -> Result<Option<QuestStateEntity>>;
    ///
    /// Retrieves the player's current level. Players level up when they complete monster quests.
    /// 
    async fn get_pl_lvl(&self, user_id: i64) -> Result<i64>;
    ///
    /// Retrieves whether the player has completed a riddle quest today
    /// 
    async fn pl_answered_riddle(&self, user_id: i64) -> Result<bool>;
    ///
    /// Creates a battle quest for the specified user, by `user_id`.
    /// If there is already an existing quest for the day and it's completed,
    /// this increments the quest level and activates it. If it's not completed
    /// returns None
    /// 
    async fn create_new_user_quest(&self, user_id: i64, quest_type: i64) -> Result<Option<QuestStateEntity>>;
    ///
    /// Completes the quest of the user with the given `user_id`
    /// 
    async fn complete_quest(&self, user_id: i64) -> Result<()>;
    ///
    /// Exhausts the player, preventing them from performing any more battle quests that day
    /// 
    async fn exhaust_pl(&self, user_id: i64) -> Result<()>;
    ///
    /// Returns whether the player is exhausted today
    /// 
    async fn pl_is_exhausted(&self, user_id: i64) -> Result<bool>;
    ///
    /// Creates a new monster with the specified stats, and assigns to the given quest
    /// 
    async fn create_quest_monster(&self, quest_id: i64, monster_idx: i64, stats: BaseStats) -> Result<()>;
    ///
    /// Retrieves the indices of all riddles the user has answered
    /// 
    async fn get_user_answered_riddle(&self, user_id: i64) -> Result<Vec<i64>>;
    ///
    /// Creates a new riddle with the specified index, and assigns to the given quest
    /// 
    async fn create_quest_riddle(&self, quest_id: i64, riddle_idx: i64) -> Result<()>;
    ///
    /// Retrieves the index of the current user's quest's riddle
    /// 
    async fn get_quest_riddle_idx(&self, user_id: i64) -> Result<Option<i64>>;
    /// 
    /// Retrieves a new, random evidence card, if any exist that has yet to be confirmed
    /// in the user's collection
    /// 
    async fn get_rand_unconfirmed_card<'a>(&self, user_id: i64, all_cards: &'a [EvidenceCardCategories]) -> Result<Option<CardModel>>;
    ///
    /// Deletes the quest with the given id
    /// 
    async fn delete_quest(&self, quest_id: i64) -> Result<()>;
}

#[derive(Constructor)]
pub struct DbQuestDataLayer {
    db: SqlitePool
}

#[async_trait]
impl QuestDataLayer for DbQuestDataLayer {
    async fn pl_has_won_game(&self, user_id: i64) -> Result<bool> {
        Ok(
            sqlx::query!("SELECT * FROM game_winners WHERE user_id = ?", user_id)
                .fetch_optional(&self.db).await?.is_some()
        )
    }
    async fn get_active_user_quest(&self, user_id: i64) -> Result<Option<QuestStateEntity>> {
        // Get the most recent, incomplete quest for the user (if one exists)
        let quest = sqlx::query!("
            SELECT id, created_on, quest_type, completed FROM quests 
            WHERE user_id = ? AND completed = FALSE ORDER BY created_on DESC
            ", user_id
        ).fetch_optional(&self.db).await?;

        if let Some(quest) = quest {
            // Check if the quest is from today - if not, return None. Otherwise, return the quest
            let today = Utc::now().naive_utc();
            if quest.created_on.num_days_from_ce() == today.num_days_from_ce() {

                // Get the monster state for the quest if it's a monster quest
                let monster_state = sqlx::query!("
                    SELECT ms.monster_idx, ms.stats_id, s.health, s.power, s.armor, s.missing_next_turn 
                    FROM monster_states ms JOIN stats s ON ms.stats_id = s.id
                    WHERE quest_id = ?
                    ", quest.id
                )
                    .fetch_optional(&self.db).await?
                    .and_then(|row| Some(QuestMonsterEntity {
                        monster_idx: row.monster_idx,
                        stats: Stats::new(row.health, row.power, row.armor, row.missing_next_turn)
                    }));

                // Get the riddle idx for the quest if it's a riddle quest
                let riddle_idx = sqlx::query!("SELECT riddle_idx FROM quest_riddles WHERE quest_id = ?", quest.id)
                    .fetch_optional(&self.db).await?
                    .and_then(|row| Some(row.riddle_idx));

                return Ok(Some(QuestStateEntity { 
                    id: quest.id, quest_type: quest.quest_type,
                    monster_state, riddle_idx,
                    completed: quest.completed
                }));
            }
        }

        Ok(None)
    }
    async fn create_new_user_quest(&self, user_id: i64, quest_type: i64) -> Result<Option<QuestStateEntity>> {
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
        sqlx::query!("INSERT INTO quests (user_id, quest_type) VALUES (?, ?)", user_id, quest_type)
            .execute(&self.db).await?;

        Ok(Some(self.get_active_user_quest(user_id).await?.unwrap()))
    }

    async fn get_pl_lvl(&self, user_id: i64) -> Result<i64> {
        Ok(
            sqlx::query!("SELECT lvl FROM users WHERE id = ?", user_id)
                .fetch_one(&self.db).await?.lvl
        )
    }

    async fn pl_answered_riddle(&self, user_id: i64) -> Result<bool> {
        Ok(
            sqlx::query!("SELECT riddle_quest_completed FROM users WHERE id = ?", user_id)
                .fetch_one(&self.db).await?.riddle_quest_completed
        )
    }

    async fn create_quest_monster(&self, quest_id: i64, monster_idx: i64, stats: BaseStats) -> Result<()> {
        let stats_id = sqlx::query!(
            "INSERT INTO stats (health, armor, missing_next_turn) VALUES (?, ?, FALSE)", 
            stats.health, stats.armor
        ).execute(&self.db).await?.last_insert_rowid();
        
        sqlx::query!("
            INSERT INTO monster_states (monster_idx, quest_id, stats_id)
            VALUES (?, ?, ?)
            ", monster_idx, quest_id, stats_id
        ).execute(&self.db).await?;

        Ok(())
    }

    async fn get_user_answered_riddle(&self, user_id: i64) -> Result<Vec<i64>> {
        // Get all user completed quests that are riddle quests
        let riddle_idxs: Vec<i64> = sqlx::query!("
            SELECT qr.riddle_idx FROM quests q JOIN quest_riddles qr ON q.id = qr.quest_id 
            WHERE q.user_id = ? AND q.quest_type = 1 AND q.completed = TRUE
            ", user_id
        )
            .fetch_all(&self.db).await?
            .iter().map(|q| q.riddle_idx).collect();

        Ok(riddle_idxs)
    }

    async fn create_quest_riddle(&self, quest_id: i64, riddle_idx: i64) -> Result<()> {
        sqlx::query!(
            "INSERT INTO quest_riddles (quest_id, riddle_idx) VALUES (?, ?)", 
            quest_id, riddle_idx
        )
            .execute(&self.db).await?;
        Ok(())
    }

    async fn get_quest_riddle_idx(&self, user_id: i64) -> Result<Option<i64>> {
        let quest = self.get_active_user_quest(user_id).await?;
        if let Some(quest) = quest {
            if quest.quest_type == 1 {
                let riddle_idx = sqlx::query!("SELECT riddle_idx FROM quest_riddles WHERE quest_id = ?", quest.id)
                    .fetch_one(&self.db).await?.riddle_idx;

                return Ok(Some(riddle_idx));
            }
        }
        Ok(None)
    }

    async fn complete_quest(&self, user_id: i64) -> Result<()> {
        // Get the quest's id
        let quest = sqlx::query!(
            "SELECT id, quest_type FROM quests WHERE user_id = ? AND completed = FALSE", 
            user_id
        )  
            .fetch_one(&self.db).await?;

        // Update the quest as completed
        sqlx::query!("UPDATE quests SET completed = TRUE WHERE id = ?", quest.id)
            .execute(&self.db).await?;

        if quest.quest_type == 0 {
            // If it was a monster battle, check if the user's lvl is currently 2
            // if so, set the player to exhausted
            let lvl = sqlx::query!("SELECT lvl FROM users WHERE id = ?", user_id)
                .fetch_one(&self.db).await?.lvl;
            if lvl == 2 {
                self.exhaust_pl(user_id).await?;
            } else {
                sqlx::query!("
                    UPDATE stats SET health = 10 
                    WHERE EXISTS (
                        SELECT * FROM user_states
                        WHERE stats_id = stats.id AND user_id = ?
                    )
                ", user_id).execute(&self.db).await?;
            }

            // If it was a monster battle, set the user's lvl to 2
            sqlx::query!("UPDATE users SET lvl = 2 WHERE id = ?", user_id)
                .execute(&self.db).await?;
        } else {
            // If it was a riddle quest, update the user as completed a riddle today
            sqlx::query!("UPDATE users SET riddle_quest_completed = TRUE WHERE id = ?", user_id)
                .execute(&self.db).await?;
        }


        Ok(())
    }

    async fn exhaust_pl(&self, user_id: i64) -> Result<()> {
        sqlx::query!("UPDATE users SET exhausted = TRUE WHERE id = ?", user_id)
            .execute(&self.db).await?;
        Ok(())
    }

    async fn pl_is_exhausted(&self, user_id: i64) -> Result<bool> {
        Ok(
            sqlx::query!("SELECT exhausted FROM users WHERE id = ?", user_id)
                .fetch_one(&self.db).await?.exhausted
        )
    }

    async fn get_rand_unconfirmed_card<'a>(&self, user_id: i64, cards: &'a [EvidenceCardCategories]) -> Result<Option<CardModel>> {
        let mut rng = StdRng::from_entropy();

        // Create an iterator of all permutations of cat idx to card idx
        let card_cat_pairs = cards.iter().enumerate()
            .flat_map(|(cat_idx, cat)| (0..cat.cards.len()).map(move |card_idx| (cat_idx as i64, card_idx as i64)));

        // Get all confirmed user cards, convert into 2-ples of cat and card idxs
        // and collect into a HashSet
        let mut conf_cat_card_idxs = sqlx::query!(
            "SELECT cat_idx, card_idx FROM user_cards WHERE user_id = ? AND confirmed = TRUE", 
            user_id
        )
            .fetch_all(&self.db).await?
            .iter().map(|card| (card.cat_idx, card.card_idx))
            .collect::<HashSet<(i64, i64)>>();
        
        // Map game target cards into confirmed cards - they should
        // never be chosen!
        let game_target_cards = sqlx::query!("SELECT * FROM game_target_cards").fetch_all(&self.db).await?;
        for idxs in game_target_cards.iter() {
            conf_cat_card_idxs.insert((idxs.cat_idx, idxs.card_idx));
        }
        
        // Choose a random cat and card idx that isn't in the confirmed collection
        let choice = card_cat_pairs.filter(|pair| !conf_cat_card_idxs.contains(pair)).choose(&mut rng);
        Ok(choice.and_then(|choice| Some(CardModel { cat_idx: choice.0, card_idx: choice.1 })))
    } 

    async fn delete_quest(&self, quest_id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM quests WHERE id = ?", quest_id)
            .execute(&self.db).await?;
        Ok(())
    }
}