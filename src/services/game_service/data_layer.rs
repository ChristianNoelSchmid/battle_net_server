use std::sync::Arc;

use axum::async_trait;
use derive_more::Constructor;
use prisma_client_rust::Direction;
use rand::{seq::SliceRandom, thread_rng};

use crate::{data_layer_error::Result, prisma::{PrismaClient, user, stats, game_winner, game_target_card, user_card}, resources::game_resources::BaseStats};

use super::models::{CardModel, MurderedUserModel, GameStateModel, UserCardModel};

const PERSON_CAT_IDX: i32 = 0;

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
    async fn setup_game<'a>(&self, target_cards: &'a [CardModel], base_stats: &'a BaseStats) -> Result<Option<MurderedUserModel>>;
    /// 
    /// Returns all current game state data, as it pertains to the particular user
    /// (ie. if the user has won, their collection of evidence cards, etc.)
    /// 
    async fn game_state(&self, user_id: i32) -> Result<Option<GameStateModel>>;
    ///
    /// Gets all target cards in the current game.
    /// The cards users have to guess to win.
    /// 
    async fn get_target_cards(&self) -> Result<Vec<CardModel>>;
    ///
    /// Assigns the user to the winner collection
    /// 
    async fn add_new_winner(&self, user_id: i32) -> Result<()>;
    ///
    /// Updates the status of a user's card, if required.
    /// Unconfirmed cards which are no longer guessed are removed, while
    /// Unconfirmed cards that are guessed are added to the user's data (if needed)
    /// 
    async fn update_user_card(&self, user_id: i32, cat_idx: i32, card_idx: i32, guessed: bool) -> Result<()>;
    ///
    /// Retrieves the indices of all evidence cards that the user has confirmed
    /// 
    async fn get_confirmed_user_cards(&self, user_id: i32) -> Result<Vec<CardModel>>;
    ///
    /// Adds card to user's confirmed set, marked as `confirmed`
    /// 
    async fn confirm_user_card(&self, user_id: i32, cat_idx: i32, card_idx: i32) -> Result<()>;
}

#[derive(Constructor)]
pub struct DbGameDataLayer { 
    db: Arc<PrismaClient>
}

#[async_trait]
impl GameDataLayer for DbGameDataLayer {
    async fn is_game_active(&self) -> Result<bool> {
        // Get the game state from the database (there should only be one)
        let game_state = self.db.game_state().find_first(vec![]).exec().await.map_err(|e| Box::new(e))?;
        
        return match game_state {
            Some(_) => Ok(true),
            None => Ok(false)
        };
    }

    async fn setup_game<'a>(&self, target_cards: &'a [CardModel], base_stats: &'a BaseStats) -> Result<Option<MurderedUserModel>> {
        // Get all user ids and card_idxs
        let users = self.db.user().find_many(vec![]).exec().await.map_err(|e| Box::new(e))?;

        // Choose a random user from the list
        let murdered_user = users.choose(&mut thread_rng());

        // Return None if there is no user to murder. There must be users present to initialize game
        if let None = murdered_user {
            return Ok(None);
        }
        let murdered_user = murdered_user.unwrap();

        // Get all other user ids
        let user_ids: Vec<i32> = users.iter().filter(|u| u.id != murdered_user.id).map(|u| u.id).collect();

        // Add the murdered user to all users evidence cards (that user is not a target card)
        for id in &user_ids {
            self.db.user_card().create(
                PERSON_CAT_IDX, murdered_user.card_idx, user::id::equals(*id), 
                vec![user_card::confirmed::set(true)]
            ).exec().await.map_err(|e| Box::new(e))?;
        }

        // Insert base player stats for each user
        for id in user_ids {
            // Create the user's stats and add to the database
            let stats = self.db.stats().create(
                base_stats.health, base_stats.armor, false, vec![]
            ).exec().await.map_err(|e| Box::new(e))?;

            // Associate the user to their stats
            self.db.user_state().create(user::id::equals(id), stats::id::equals(stats.id), vec![])
                .exec().await.map_err(|e| Box::new(e))?;
        }

        // Insert each generated target card into the game_target_cards table
        for target_card in target_cards {
            self.db.game_target_card().create(target_card.cat_idx, target_card.card_idx, vec![])
                .exec().await.map_err(|e| Box::new(e))?;
        }

        // Add the initialized game state
        self.db.game_state().create(user::id::equals(murdered_user.id), vec![])
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(Some(MurderedUserModel { card_idx: murdered_user.card_idx }))
    }

    async fn game_state(&self, user_id: i32) -> Result<Option<GameStateModel>> {
        // Determine if the given user has won the game
        let has_won = self.db.game_winner().find_first(vec![game_winner::user_id::equals(user_id)])
            .exec().await.map_err(|e| Box::new(e))?;

        // If the user has won, retrieve the target cards and all current winners
        let (mut target_cards, mut winner_idxs) = (None, None);

        if let Some(_) = has_won {
            target_cards = Some(
                self.db.game_target_card().find_many(vec![]).order_by(game_target_card::OrderByParam::CatIdx(Direction::Asc))
                    .exec().await.map_err(|e| Box::new(e))?
                    .iter().map(|card| CardModel { cat_idx: card.cat_idx, card_idx: card.card_idx }).collect()
            );
            
            winner_idxs = Some(
                self.db.game_winner().find_many(vec![]).order_by(game_winner::OrderByParam::Id(Direction::Asc))
                    .with(game_winner::user::fetch())
                    .exec().await.map_err(|e| Box::new(e))?
                    .iter().map(|r| r.user.as_ref().unwrap().card_idx).collect()
            );
        }

        // Get the user's current guessed cards and confirmed cards
        let user_cards = self.db.user_card().find_many(vec![user_card::user_id::equals(user_id)])
            .exec().await.map_err(|e| Box::new(e))?
            .iter().map(|card| UserCardModel { cat_idx: card.cat_idx, card_idx: card.card_idx, confirmed: card.confirmed })
            .collect();
        
        let game_state = self.db.game_state().find_first(vec![])
            .exec().await.map_err(|e| Box::new(e))?;

        return match game_state {
            Some(game_state) => Ok(Some(GameStateModel { 
                murdered_user_idx: game_state.murdered_user_id, 
                target_cards, 
                user_cards, 
                winner_idxs 
            })),
            None => Ok(None)
        };
    }

    async fn get_target_cards(&self) -> Result<Vec<CardModel>> {
        Ok(
            self.db.game_target_card().find_many(vec![]).exec().await.map_err(|e| Box::new(e))?
                .iter().map(|card| CardModel { cat_idx: card.cat_idx, card_idx: card.card_idx }).collect()
        )
    }

    async fn add_new_winner(&self, user_id: i32) -> Result<()> {
        self.db.game_winner().create(user::id::equals(user_id), vec![]).exec().await.map_err(|e| Box::new(e))?;
        Ok(())
    }

    async fn update_user_card(&self, user_id: i32, cat_idx: i32, card_idx: i32, guessed: bool) -> Result<()> {

        // Get the current value of the choice card
        let user_card = self.db.user_card().find_first(vec![
            user_card::cat_idx::equals(cat_idx), 
            user_card::card_idx::equals(card_idx), 
            user_card::user_id::equals(user_id)
        ])
            .exec().await.map_err(|e| Box::new(e))?;

        match user_card {
            // If there's no user card that matches and the user has guessed, insert
            // that card, unconfirmed
            None if guessed => {
                self.db.user_card().create(cat_idx, card_idx, user::id::equals(user_id), vec![])
                    .exec().await.map_err(|e| Box::new(e))?;
            },
            // If a card does exist, is unconfirmed, and the user no longer guesses it,
            // delete that card
            Some(user_card) if !user_card.confirmed && !guessed => {
                self.db.user_card().delete(user_card::UniqueWhereParam::UserIdCatIdxCardIdxEquals(user_id, cat_idx, card_idx))
                    .exec().await.map_err(|e| Box::new(e))?;
            },
            // Ignore any other cases - they do not affect the user card
            // (card exists and is guessed doesn't matter, as there is already a card representing this)
            // (card confirmed but no longer guessed doesn't matter, as the card is confirmed)
            _ => {}
        }

        Ok(())
    }

    async fn get_confirmed_user_cards(&self, user_id: i32) -> Result<Vec<CardModel>> {
        Ok(
            self.db.user_card().find_many(vec![user_card::user_id::equals(user_id), user_card::confirmed::equals(true)])
            .exec().await.map_err(|e| Box::new(e))?
            .iter().map(|card| CardModel { cat_idx: card.cat_idx, card_idx: card.card_idx }).collect()
        )
    }

    async fn confirm_user_card(&self, user_id: i32, cat_idx: i32, card_idx: i32) -> Result<()> {
        // If the card already exists in the users evidence cards, update it to confirmed
        if self.get_confirmed_user_cards(user_id).await?.iter().any(|c| c == &CardModel { cat_idx, card_idx }) {
            self.db.user_card().update(
                user_card::UniqueWhereParam::UserIdCatIdxCardIdxEquals(user_id, cat_idx, card_idx),
                vec![user_card::confirmed::set(true)]
            ).exec().await.map_err(|e| Box::new(e))?;

        // Otherwise, create a new user card as confirmed
        } else {
            self.db.user_card().create(cat_idx, card_idx, user::id::equals(user_id), vec![user_card::confirmed::set(true)])
                .exec().await.map_err(|e| Box::new(e))?;
        }

        Ok(())
    } 
}