use std::sync::Arc;

use axum::{Router, routing::{post}, extract::{FromRef, Path, State}, Json, middleware};

use crate::{middleware::auth_middleware::{AuthContext, auth_middleware}, services::{quest_service::{error::Result, QuestService}, token_service::TokenService}, models::quest_models::{QuestModel, RiddleStatus}};

#[derive(Clone, FromRef)]
pub struct QuestRoutesState {
    quest_service: Arc<dyn QuestService>
}

pub fn routes(quest_service: Arc<dyn QuestService>, token_service: Arc<dyn TokenService>) -> Router {
    Router::new()
        // Routes
        .route("/create/:typ", post(create_quest))
        .route("/guess-riddle/:answer", post(guess_riddle))
        // Auth middleware
        .layer(middleware::from_fn_with_state(token_service, auth_middleware))
        // State
        .with_state(QuestRoutesState { quest_service })
}

async fn create_quest(
    State(quest_service): State<Arc<dyn QuestService>>,
    Path(typ): Path<i32>,
    ctx: AuthContext,
) -> Result<Json<QuestModel>> {
    Ok(Json(quest_service.generate_quest(ctx.user_id, typ).await?))
}

async fn guess_riddle(
    State(quest_service): State<Arc<dyn QuestService>>,
    Path(answer): Path<String>,
    ctx: AuthContext,
) -> Result<Json<RiddleStatus>> {
    Ok(Json(quest_service.guess_riddle(ctx.user_id, answer).await?))
}
