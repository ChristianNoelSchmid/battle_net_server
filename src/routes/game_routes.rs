use std::sync::Arc;

use axum::{Router, routing::{post, get}, extract::{State, FromRef}, Json, middleware};

use crate::{
    services::{game_service::{GameService, error::Result, models::{GameInitialStateModel, GameStateModel, UserCardModel}}, token_service::TokenService},
    middleware::auth_middleware::{AuthContext, auth_middleware, AdminContext},
};

#[derive(Clone, FromRef)]
pub struct GameRoutesState {
    game_service: Arc<dyn GameService>,
}

pub fn routes(game_service: Arc<dyn GameService>, token_service: Arc<dyn TokenService>) -> Router {
    let router = Router::new()
        // Routes
        .route("/setup", post(setup_game))
        .route("/state", get(game_state))
        .route("/guess", post(guess_target_cards))
        .route("/update-card", post(update_user_card))
        // Auth middleware
        .layer(middleware::from_fn_with_state(token_service, auth_middleware))
        // State
        .with_state(GameRoutesState { game_service });

    router
}

async fn setup_game(State(game_service): State<Arc<dyn GameService>>, _admin: AdminContext) -> Result<Json<GameInitialStateModel>> {
    Ok(Json(game_service.setup_game().await?))
}

async fn game_state(State(game_service): State<Arc<dyn GameService>>, ctx: AuthContext) -> Result<Json<GameStateModel>> {
    Ok(Json(game_service.game_state(ctx.user_id).await?))
}

async fn guess_target_cards(State(game_service): State<Arc<dyn GameService>>, ctx: AuthContext, guess: Json<Vec<i32>>) -> Result<Json<bool>> {
    Ok(Json(game_service.guess_target_cards(ctx.user_id, &guess).await?))
}

async fn update_user_card(State(game_service): State<Arc<dyn GameService>>, ctx: AuthContext, card: Json<UserCardModel>) -> Result<()> {
    game_service.update_user_card(ctx.user_id, card.cat_idx, card.card_idx, card.confirmed).await?;
    Ok(())
}