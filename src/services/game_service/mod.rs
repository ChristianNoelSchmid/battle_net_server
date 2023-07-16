pub mod error;
pub mod data_layer;
pub mod dtos;

use std::sync::Arc;

use axum::async_trait;
use derive_more::Constructor;
use rand::{seq::IteratorRandom, rngs::StdRng, SeedableRng};

use crate::resources::game_resources::Resources;

use self::{error::{GameServiceError, Result}, data_layer::{GameDataLayer, entities::CardModel}, dtos::{GameStateDto, GameInitialStateDto}};

use super::auth_service::AuthService;

#[async_trait]
pub trait GameService : Send + Sync {
    async fn setup_game(&self,) -> Result<GameInitialStateDto>;
    async fn game_state<'a>(&self, usr_id: i64,) -> Result<GameStateDto>;
    async fn guess_target_cards<'a>(&self, user_id: i64, guess: &'a [i64],) -> Result<bool>;
}

#[derive(Constructor)]
pub struct DbGameService { 
    data_layer: Arc<dyn GameDataLayer>,
    auth_service: Arc<dyn AuthService>,
    res: Arc<Resources>
}

#[async_trait]
impl GameService for DbGameService {
    async fn setup_game(&self) -> Result<GameInitialStateDto> {
        let mut rng = StdRng::from_entropy();

        // Get the base user stats
        let ubs = self.res.user_base_stats;

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

        let murdered_user = self.data_layer.setup_game(&target_cards, &ubs).await.map_err(|e| e.into())?;

        Ok(GameInitialStateDto {
            target_cards,
            murdered_user_id: murdered_user.id,
        })
    }

    async fn game_state<'a>(&self, user_id: i64,) -> Result<GameStateDto> {
        let state_model = self.data_layer.game_state(user_id).await.map_err(|e| e.into())?;
        return if let Some(m) = state_model {
            Ok(GameStateDto { 
                evd_cats_and_cards: self.res.evd_cats_and_cards.clone(), 
                murdered_user_idx: m.murdered_user_idx, 
                target_card_idxs: m.target_cards,
                winner_ids: m.winner_idxs 
            })
        } else {
            Err(GameServiceError::GameNotRunning)
        }
    }

    async fn guess_target_cards<'a>(&self, user_id: i64, guess: &'a [i64]) -> Result<bool> {
        let target_cards = self.data_layer.get_target_cards().await.map_err(|e| e.into())?;

        if let Some(mut target_cards) = target_cards {
            // Sort the target cards by category index (expected order of request guess)
            target_cards.sort_by(|a, b| a.cat_idx.partial_cmp(&b.cat_idx).unwrap());
            // If the lengths do not match, the guess is incorrect
            if guess.len() != target_cards.len() {
                return Ok(false);
            }
            // If any target card index does not match the guess card index, 
            // the guess is incorrect
            for pair in target_cards.iter().zip(guess) {
                if &pair.0.card_idx != pair.1 {
                    return Ok(false);
                }
            }
        }

        // Otherwise, guess is correct - insert user as new winner 
        self.data_layer.add_new_winner(user_id).await.map_err(|e| e.into())?;
        Ok(true)
    }
}

impl DbGameService {
    async fn insert_test_data(&self) -> Result<()> {
        let users = [
            (0, "chris@mail.com",  "ChrisSchmid"),
            (1, "alyssaq@mail.com", "AlyssaSchmid"),
            (2, "andrea@mail.com", "AndreaBuckalew"),
            (3, "zach@mail.com", "Zach Buckalew"),
            (4, "alyssac@mail.com", "AlyssaHillen"),
            (5, "kunane@mail.com", "KunaneHillen"),
            (6, "brian@mail.com", "BrianHall"),
            (7, "miranda@mail.com", "MirandaHall"),
            (8, "maria@mail.com", "MariaMcGowan"),
            (9, "mj@mail.com", "MJSchmid"),
            (10, "kim@mail.com", "KimSchmid")
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

