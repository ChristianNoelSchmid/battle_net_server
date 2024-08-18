use std::{sync::Arc, ops::ControlFlow};

use axum::{Router, routing::get, middleware, extract::{ws::{Message, WebSocket, WebSocketUpgrade}, FromRef, State}, response::IntoResponse, http::StatusCode};
use log::error;

use crate::{services::{token_service::TokenService, battle_service::BattleService, quest_service::{QuestService, error::QuestServiceError}}, middleware::auth_middleware::{auth_middleware, AuthContext}};

#[derive(Clone, FromRef)]
pub struct BattleRoutesState {
    battle_service: Arc<dyn BattleService>,
    quest_service: Arc<dyn QuestService>,
}

pub fn routes(token_service: Arc<dyn TokenService>, quest_service: Arc<dyn QuestService>, battle_service: Arc<dyn BattleService>) -> Router {
    Router::new()
        // Routes
        .route("/", get(ws_handler))
        // Auth middleware
        .layer(middleware::from_fn_with_state(token_service, auth_middleware))
        // State
        .with_state(BattleRoutesState { battle_service, quest_service })
}

/// The handler for the HTTP request (this gets called when the HTTP GET lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
async fn ws_handler(
    ws: WebSocketUpgrade,
    ctx: AuthContext,
    State(battle_service): State<Arc<dyn BattleService>>,
    State(quest_service): State<Arc<dyn QuestService>>
) -> impl IntoResponse {
    // Ensure the user is currently on a quest - if not, return BAD REQUEST
    match quest_service.get_quest(ctx.user_id).await {
        Err(QuestServiceError::UserNotOnQuest) => {
            return (StatusCode::BAD_REQUEST, "User does not have a quest active").into_response();
        }
        Err(e) => { return e.into_response(); },
        Ok(quest) => {
            if quest.monster_state.is_none() {
                return (StatusCode::BAD_REQUEST, "Only battles use websocket connections").into_response()
            }
        }
    }
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, ctx.user_id, battle_service))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(mut socket: WebSocket, user_id: i64, battle_service: Arc<dyn BattleService>) {
    // Send a Ping message, and return if error occurs (ie. client disconnects immediately)
    let setup = battle_service.setup(user_id).await;

    if let Err(e) = setup { 
        error!("{:?}", e);
        socket.send(Message::Text("INTERNAL SERVER ERROR".to_string())).await.ok();
        return;
    }
    if socket.send(setup.unwrap().to_ws_msg()).await.is_err() {
        return;
    }

    while let Some(Ok(msg)) = socket.recv().await {
        match process_message(msg, user_id, battle_service.clone()).await {
            ControlFlow::Continue(Some(msg)) => { 
                if let Err(e) = socket.send(msg).await {
                    error!("Error sending message to user_id {user_id}: `{:?}`", e);
                    break;
                } 
            },
            ControlFlow::Break(Some(msg)) => {
                if let Err(e) = socket.send(msg).await {
                    error!("Error sending message to user_id {user_id}: `{:?}", e);
                }
                // Break after sending the socket message (if one exists)
                break;
            },
            _ => { }
        }
    }

    socket.close().await.unwrap();
}

/// helper to print contents of messages to stdout. Has special treatment for Close.
async fn process_message(msg: Message, user_id: i64, battle_service: Arc<dyn BattleService>) -> ControlFlow<Option<Message>, Option<Message>> {
    match msg {
        Message::Text(t) => {
            // Parse the client command from the possible choices in the battle
            if let Some((cmd, msg)) = t.split_once("::") {
                // If command contains an argument, ensure it is a i64
                if let Ok(val) = msg.parse::<i64>() {
                    match cmd {
                        "Attack" => {
                            match battle_service.attack(user_id, val).await {
                                Ok(round_res) => {
                                    return if round_res.battle_completed() {
                                        ControlFlow::Break(Some(round_res.to_ws_msg()))
                                    } else {
                                        ControlFlow::Continue(Some(round_res.to_ws_msg()))
                                    };
                                },
                                Err(e) => return ControlFlow::Continue(Some(Message::Text(e.to_string())))
                            }
                        }
                        "Item" => {
                            match battle_service.use_item(user_id, val).await {
                                Ok(round_res) => {
                                    return if round_res.battle_completed() {
                                        ControlFlow::Break(Some(round_res.to_ws_msg()))
                                    } else {
                                        ControlFlow::Continue(Some(round_res.to_ws_msg()))
                                    };
                                },
                                Err(e) => return ControlFlow::Continue(Some(Message::Text(e.to_string())))
                            }
                        }
                        _ => {
                            let err = format!("Could not understand command `{cmd}`");
                            return ControlFlow::Continue(Some(Message::Text(err)));
                        }
                    }
                } else {
                    let err = format!("Command `{cmd}` had non-uint parameter `{msg}`");
                    return ControlFlow::Continue(Some(Message::Text(err)));
                }
            } else {
                // If the command does not have arguments
                match t.as_ref() {
                    "Defend" => {
                        match battle_service.defend(user_id).await {
                            Ok(round_res) => return if round_res.battle_completed() {
                                ControlFlow::Break(Some(round_res.to_ws_msg()))
                            } else {
                                ControlFlow::Continue(Some(round_res.to_ws_msg()))
                            },
                            Err(e) => return ControlFlow::Continue(Some(Message::Text(e.to_string())))
                        }
                    },
                    _ => {
                        let err = format!("Could not understand command `{t}`");
                        return ControlFlow::Continue(Some(Message::Text(err)));
                    }
                }
            }
        }
        Message::Close(_) => return ControlFlow::Break(None),
        _ => { }
    }
    ControlFlow::Continue(None)
}