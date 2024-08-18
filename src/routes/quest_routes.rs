use std::sync::Arc;

use axum::{Router, routing::{post, get}, extract::{FromRef, Path, State}, Json, middleware};

use crate::{middleware::auth_middleware::{AuthContext, auth_middleware}, services::{quest_service::{error::Result, QuestService, models::{QuestStateModel, RiddleStatus}}, token_service::TokenService}};

#[derive(Clone, FromRef)]
pub struct QuestRoutesState {
    quest_service: Arc<dyn QuestService>
}

pub fn routes(quest_service: Arc<dyn QuestService>, token_service: Arc<dyn TokenService>) -> Router {
    Router::new()
        // Routes
        .route("/create/:typ", post(create_quest))
        .route("/guess-riddle/:answer", post(guess_riddle))
        .route("/current", get(get_quest))
        // Auth middleware
        .layer(middleware::from_fn_with_state(token_service, auth_middleware))
        // State
        .with_state(QuestRoutesState { quest_service })
}

async fn create_quest(
    State(quest_service): State<Arc<dyn QuestService>>,
    Path(typ): Path<i64>,
    ctx: AuthContext,
) -> Result<Json<QuestStateModel>> {
    Ok(Json(quest_service.generate_quest(ctx.user_id, typ).await?))
}

async fn guess_riddle(
    State(quest_service): State<Arc<dyn QuestService>>,
    Path(answer): Path<String>,
    ctx: AuthContext,
) -> Result<Json<RiddleStatus>> {
    Ok(Json(quest_service.guess_riddle(ctx.user_id, answer).await?))
}

async fn get_quest(
    State(quest_service): State<Arc<dyn QuestService>>,
    ctx: AuthContext
) -> Result<Json<QuestStateModel>> {
    Ok(Json(quest_service.get_quest(ctx.user_id).await?))
}
