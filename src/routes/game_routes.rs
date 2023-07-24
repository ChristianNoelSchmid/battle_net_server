use std::sync::Arc;

use axum::{Router, routing::{post, get}, extract::{State, FromRef}, Json, middleware};

use crate::{
    services::{game_service::{GameService, error::Result}, token_service::TokenService},
    resources::game_resources::Resources, middleware::auth_middleware::{AuthContext, auth_middleware, AdminContext}, models::game_models::{GameStateModel, GameInitialStateModel, UserCardModel},
};

#[derive(Clone, FromRef)]
pub struct GameRoutesState {
    res: Arc<Resources>,
    game_service: Arc<dyn GameService>,
}

pub fn routes(game_service: Arc<dyn GameService>, token_service: Arc<dyn TokenService>, res: Arc<Resources>) -> Router {
    let router = Router::new()
        // Routes
        .route("/setup", post(setup_game))
        .route("/state", get(game_state))
        .route("/guess", post(guess_target_cards))
        .route("/update-card", post(update_user_card))
        // Auth middleware
        .layer(middleware::from_fn_with_state(token_service, auth_middleware))
        // State
        .with_state(GameRoutesState { res, game_service });

    router
}

async fn setup_game(State(state): State<GameRoutesState>, _admin: AdminContext) -> Result<Json<GameInitialStateModel>> {
    Ok(Json(state.game_service.setup_game().await?))
}

async fn game_state(State(state): State<GameRoutesState>, ctx: AuthContext) -> Result<Json<GameStateModel>> {
    Ok(Json(state.game_service.game_state(ctx.user_id).await?))
}

async fn guess_target_cards(State(state): State<GameRoutesState>, ctx: AuthContext, guess: Json<Vec<i32>>) -> Result<Json<bool>> {
    Ok(Json(state.game_service.guess_target_cards(ctx.user_id, &guess).await?))
}

async fn update_user_card(State(state): State<GameRoutesState>, ctx: AuthContext, card: Json<UserCardModel>) -> Result<()> {
    state.game_service.update_user_card(ctx.user_id, card.cat_idx, card.card_idx, card.confirmed).await?;
    Ok(())
}