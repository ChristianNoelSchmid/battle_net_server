pub mod error;
pub mod data_layer;
pub mod models;

use std::sync::Arc;

use axum::async_trait;
use derive_more::Constructor;
use models::GuessResult;
use rand::{seq::IteratorRandom, rngs::StdRng, SeedableRng};

use crate::resources::game_resources::Resources;

use self::{error::{GameServiceError, Result}, data_layer::GameDataLayer, models::{CardModel, GameInitialStateModel, GameStateModel}};

use super::auth_service::AuthService;

///
/// Service which interacts with the endgame components
/// of the game, such as setting up, retrieving state,
/// guessing the target cards and updating user cards with
/// guesses.
/// 
#[async_trait]
pub trait GameService : Send + Sync {
    ///
    /// Establishes a new game, if one does not already exist.
    /// Returns `GameServiceError::GameAlreadyRunning` if game has
    /// already been set up
    /// 
    async fn setup_game(&self,) -> Result<GameInitialStateModel>;
    ///
    /// Retrieves the state of the game, including user-specific 
    /// state.
    /// 
    async fn game_state<'a>(&self, usr_id: i64) -> Result<GameStateModel>;
    ///
    /// Allows the user to guess the target cards with the provided slice of `i64`s.
    /// Returns `true` if the cards are correctly guessed, `false` otherwise.
    /// Guesses are in order of category ID (ie. the first index must be for the 
    /// first category, etc.)
    /// 
    async fn guess_target_cards<'a>(&self, user_id: i64, guess: &'a [i64]) -> Result<GuessResult>;
    ///
    /// Updates a user's card state with the particular guess-decision of the card specified.
    /// 
    async fn update_user_card(&self, user_id: i64, cat_idx: i64, card_idx: i64, guessed: bool) -> Result<()>;
    ///
    /// Confirms a card for the given `user_id`, with specific `cat_idx` and `card_idx`
    /// 
    async fn confirm_user_card(&self, user_id: i64, cat_idx: i64, card_idx: i64) -> Result<()>;

}

///
/// Database implementation of `GameService`
/// 
#[derive(Constructor)]
pub struct DbGameService { 
    data_layer: Arc<dyn GameDataLayer>,
    auth_service: Arc<dyn AuthService>,
    res: Arc<Resources>
}

#[async_trait]
impl GameService for DbGameService {
    async fn setup_game(&self) -> Result<GameInitialStateModel> {
        let mut rng = StdRng::from_entropy();

        // Ensure there is no game actively running
        if self.data_layer.is_game_active().await.map_err(|e| e.into())? {
            return Err(GameServiceError::GameAlreadyRunning);
        }

        self.insert_test_data().await?;

        // For each category, select one card as the target card
        let mut target_cards = Vec::new();
        for (cat_idx, cat) in self.res.evd_cats_and_cards.iter().enumerate() {
            let (card_idx, _) = cat.cards.iter().enumerate().choose(&mut rng).expect("Could not find all category cards.");
            target_cards.push(CardModel { cat_idx: cat_idx as i64, card_idx: card_idx as i64 });
        }

        self.data_layer.setup_game(&target_cards, &self.res.user_base_stats).await.map_err(|e| e.into())?;
        self.auth_service.print_all_access_tokens().await.map_err(|e| e.into())?;

        Ok(GameInitialStateModel { target_cards, })
    }

    async fn game_state<'a>(&self, user_id: i64) -> Result<GameStateModel> {
        let state_model = self.data_layer.game_state(user_id).await.map_err(|e| e.into())?;
        let completed_riddle_count = self.data_layer.get_completed_riddle_count(user_id).await.map_err(|e| e.into())?;
        state_model.and_then(|mut model| {
            if completed_riddle_count as usize == self.res.riddles.len() {
                model.pl_completed_all_riddles = true;
            }
            Some(model)
        }).ok_or(GameServiceError::GameNotRunning)
    }

    async fn guess_target_cards<'a>(&self, user_id: i64, guess: &'a [i64]) -> Result<GuessResult> {
        let winners = self.game_state(user_id).await?.winner_idxs;
        if winners.is_some() {
            return Ok(GuessResult::AlreadyWon);
        }
        let guessed_today = self.data_layer.pl_guessed_today(user_id).await.map_err(|e| e.into())?;
        if guessed_today {
            return Ok(GuessResult::AlreadyGuessedToday);
        }

        let mut target_cards = self.data_layer.get_target_cards().await.map_err(|e| e.into())?;

        if target_cards.is_empty() {
            return Err(GameServiceError::GameNotRunning)
        }

        self.data_layer.update_guessed_today(user_id).await.map_err(|e| e.into())?;

        // Sort the target cards by category index (expected order of request guess)
        target_cards.sort_by(|a, b| a.cat_idx.partial_cmp(&b.cat_idx).unwrap());
        // If the lengths do not match, the guess is incorrect
        if guess.len() != target_cards.len() {
            return Ok(GuessResult::Incorrect);
        }
        // If any target card index does not match the guess card index, 
        // the guess is incorrect
        if target_cards.iter().zip(guess).any(|(f, s)| f.card_idx != *s) {
            return Ok(GuessResult::Incorrect);
        }

        // Otherwise, guess is correct - insert user as new winner 
        self.data_layer.add_new_winner(user_id).await.map_err(|e| e.into())?;

        let ans = <[i64;3]>::try_from(guess);
        Ok(GuessResult::Correct(ans.unwrap()))
    }

    async fn update_user_card(&self, user_id: i64, cat_idx: i64, card_idx: i64, guessed: bool) -> Result<()> {
        if cat_idx as usize >= self.res.evd_cats_and_cards.len() || 
           card_idx as usize >= self.res.evd_cats_and_cards[cat_idx as usize].cards.len() {
            return Err(GameServiceError::GuessOutOfRange);
        }
        self.data_layer.update_user_card(user_id, cat_idx, card_idx, guessed).await.map_err(|e| e.into())?;
        Ok(())
    }
    async fn confirm_user_card(&self, user_id: i64, cat_idx: i64, card_idx: i64) -> Result<()> {
        self.data_layer.confirm_user_card(user_id, cat_idx, card_idx).await.map_err(|e| e.into())?;
        Ok(())
    }
}

impl DbGameService {
    ///
    /// Inserts testing data into the game at initial setup
    /// 
    async fn insert_test_data(&self) -> Result<()> {
        let users = [
            (0, "AlyssaC", "AlyssaHillen"),
            (1, "AlyssaQ", "AlyssaSchmid"),
            (2, "Andrea", "AndreaBuckalew"),
            (3, "Brian", "BrianHall"),
            (4, "Chris",  "ChrisSchmid"),
            (5, "Coraline", "CoralineSchmid"),
            (6, "Kim", "KimSchmid"),
            (7, "Kunane", "KunaneHillen"),
            (8, "Maria", "MariaMcGowan"),
            (9, "Miranda", "MirandaHall"),
            (10, "MJ", "MJSchmid"),
            (11, "Paisley", "PaisleySchmid"),
            (12, "Zach", "ZachBuckalew"),
        ];

        for (card_idx, email, pwd) in users.iter() {
            self.auth_service.create_new_user(
                email.to_string(), 
                pwd.to_string(), 
                *card_idx
            ).await.map_err(|e| e.into())?;
        }

        Ok(())
    }

    
}

