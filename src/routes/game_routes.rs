use std::sync::Arc;

use axum::{Router, routing::{post, get}, extract::{State, FromRef}, Json, middleware};

use crate::{
    services::{game_service::{GameService, error::Result, dtos::{GameInitialStateDto, GameStateDto}}, token_service::TokenService},
    resources::game_resources::Resources, middleware::auth_middleware::{AuthContext, auth_middleware, AdminContext},
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
        // Auth middleware
        .layer(middleware::from_fn_with_state(token_service, auth_middleware))
        // State
        .with_state(GameRoutesState { res, game_service });

    router
}

async fn setup_game(State(state): State<GameRoutesState>, _admin: AdminContext) -> Result<Json<GameInitialStateDto>> {
    Ok(Json(state.game_service.setup_game().await?))
}

async fn game_state(State(state): State<GameRoutesState>) -> Result<Json<GameStateDto>> {
    Ok(Json(state.game_service.game_state(0i64).await?))
}

async fn guess_target_cards(State(state): State<GameRoutesState>, ctx: AuthContext, guess: Json<Vec<i64>>) -> Result<Json<bool>> {
    Ok(Json(state.game_service.guess_target_cards(ctx.user_id, &guess).await?))
}
