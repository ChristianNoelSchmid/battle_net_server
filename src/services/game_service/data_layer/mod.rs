pub mod entities;

use axum::async_trait;
use derive_more::Constructor;
use sqlx::SqlitePool;

use crate::{data_layer_error::Result, resources::game_resources::BaseStats, services::token_service::TokenService};

use self::entities::{MurderedUserModel, CardModel, GameStateModel};

const PERSON_CAT_IDX: i64 = 0;

#[async_trait]
pub trait GameDataLayer : Send + Sync {
    ///
    /// Checks if there is a game currently running
    /// 
    async fn is_game_active(&self) -> Result<bool>;
    ///
    /// Sets up the game with the given target cards (excluding the murdered user card,
    /// as that is generated in this process), and base user stats. 
    /// Returns the randomly chosen murdered user.
    ///
    async fn setup_game<'a>(&self, target_cards: &'a [CardModel], base_stats: &'a BaseStats) -> Result<MurderedUserModel>;
    /// 
    /// Returns all current game state data, as it pertains to the particular user
    /// (ie. if the user has won, their collection of evidence cards, etc.)
    /// 
    async fn game_state(&self, user_id: i64) -> Result<Option<GameStateModel>>;
    ///
    /// Gets all target cards in the current game.
    /// The cards users have to guess to win.
    /// 
    async fn get_target_cards(&self) -> Result<Option<Vec<CardModel>>>;
    ///
    /// Assigns the user to the winner collection
    /// 
    async fn add_new_winner(&self, user_id: i64) -> Result<()>;
}

#[derive(Constructor)]
pub struct DbGameDataLayer { 
    db: SqlitePool,
}

#[async_trait]
impl GameDataLayer for DbGameDataLayer {
    async fn is_game_active(&self) -> Result<bool> {
        let game_state = sqlx::query!("SELECT murdered_user_id FROM game_state LIMIT 1")
            .fetch_one(&self.db).await;
        
        return match game_state {
            Ok(_) => Ok(true),
            Err(sqlx::Error::RowNotFound) => Ok(false),
            Err(e) => Err(Box::new(e))
        };
    }

    async fn setup_game<'a>(&self, target_cards: &'a [CardModel], base_stats: &'a BaseStats) -> Result<MurderedUserModel> {
        // Select a random user as the murdered user
        let murdered_user = sqlx::query_as!(MurderedUserModel, "SELECT id, card_idx FROM users ORDER BY RANDOM()")
            .fetch_one(&self.db).await.map_err(|e| Box::new(e))?;
        
        // Add the murdered user to all users evidence cards (that user is not a target card)
        let users = sqlx::query!("SELECT id FROM users").fetch_all(&self.db).await.map_err(|e| Box::new(e))?;
        for user in &users {
            sqlx::query!(r"INSERT INTO user_evidence_cards (user_id, cat_idx, card_idx, confirmed)
                VALUES (?, ?, ?, 1)",
                user.id, PERSON_CAT_IDX, murdered_user.card_idx
            ).execute(&self.db).await.map_err(|e| Box::new(e))?;
        }

        // Insert base player stats for each user
        for user in users {
            let stats = sqlx::query!(r"INSERT INTO stats (health, magicka, armor, wisdom, reflex, missing_next_turn)
                VALUES (?, ?, ?, ?, ?, FALSE)",
                base_stats.health, base_stats.magicka, base_stats.armor, base_stats.wisdom, base_stats.reflex
            ).execute(&self.db).await.map_err(|e| Box::new(e))?;
            let new_row_id = stats.last_insert_rowid();

            // Insert player state for each user
            sqlx::query!("INSERT INTO user_state (user_id, cur_stats_id) VALUES (?, ?)", user.id, new_row_id)
                .execute(&self.db).await.map_err(|e| Box::new(e))?;
        }

        // Insert each generated target card into the game_target_cards table
        for target_card in target_cards {
            sqlx::query!("INSERT INTO game_target_cards (cat_idx, card_idx) VALUES (?, ?)", target_card.cat_idx, target_card.card_idx)
                .execute(&self.db).await.map_err(|e| Box::new(e))?;
        }

        // Add the initialized game state
        sqlx::query!("INSERT INTO game_state (murdered_user_id) VALUES (?)", murdered_user.id)
            .execute(&self.db).await.map_err(|e| Box::new(e))?;

        Ok(murdered_user)
    }

    async fn game_state(&self, user_id: i64) -> Result<Option<GameStateModel>> {
        // Determine if the given user has won the game
        let has_won = sqlx::query!("SELECT * FROM game_winners WHERE user_id = ?", user_id)
            .fetch_one(&self.db).await;

        let (target_cards, winner_idxs);

        match has_won {
            // If the user has won, retrieve the target cards and all current winners
            Ok(_) => {
                target_cards = Some(
                    sqlx::query_as!(CardModel, "SELECT cat_idx, card_idx FROM game_target_cards ORDER BY cat_idx")
                        .fetch_all(&self.db).await.map_err(|e| Box::new(e))?
                );
                winner_idxs = Some(
                    sqlx::query!("SELECT u.card_idx FROM game_winners gw JOIN users u ON gw.user_id = u.id ORDER BY gw.id")
                        .fetch_all(&self.db).await.map_err(|e| Box::new(e))?
                        .iter().map(|rec| rec.card_idx)
                        .collect()
                );
            },
            // Otherwise, return None for target game cards and winners
            Err(sqlx::Error::RowNotFound) => (target_cards, winner_idxs) = (None, None),
            Err(e) => return Err(Box::new(e))
        }

        let murdered_user_idx = sqlx::query!("SELECT id FROM users JOIN game_state ON murdered_user_id = users.id")
            .fetch_one(&self.db).await;

        return match murdered_user_idx {
            Ok(rec) => Ok(Some(GameStateModel { murdered_user_idx: rec.id, target_cards, winner_idxs })),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(Box::new(e))
        };
    }

    async fn get_target_cards(&self) -> Result<Option<Vec<CardModel>>> {
        let target_cards = sqlx::query_as!(CardModel, "SELECT card_idx, cat_idx FROM game_target_cards")
            .fetch_all(&self.db).await;

        return match target_cards {
            Ok(target_cards) => Ok(Some(target_cards)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(Box::new(e))
        };
    }

    async fn add_new_winner(&self, user_id: i64) -> Result<()> {
        sqlx::query!("INSERT INTO game_winners (user_id) VALUES (?)", user_id)
            .execute(&self.db).await.map_err(|e| Box::new(e))?;

        Ok(()) 
    }
}