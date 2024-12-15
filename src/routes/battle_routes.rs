use std::{sync::Arc, ops::ControlFlow};

use axum::{Router, routing::get, extract::{ws::{Message, WebSocket, WebSocketUpgrade}, FromRef, State}, response::IntoResponse};
use log::error;

use crate::services::{battle_service::BattleService, quest_service::{error::QuestServiceError, QuestService}, token_service::TokenService};

#[derive(Clone, FromRef)]
pub struct BattleRoutesState {
    token_service: Arc<dyn TokenService>,
    battle_service: Arc<dyn BattleService>,
    quest_service: Arc<dyn QuestService>,
}

pub fn routes(token_service: Arc<dyn TokenService>, quest_service: Arc<dyn QuestService>, battle_service: Arc<dyn BattleService>) -> Router {
    Router::new()
        // Routes
        .route("/", get(ws_handler))
        // State
        .with_state(BattleRoutesState { token_service, battle_service, quest_service })
}

/// The handler for the HTTP request (this gets called when the HTTP GET lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(token_service): State<Arc<dyn TokenService>>,
    State(battle_service): State<Arc<dyn BattleService>>,
    State(quest_service): State<Arc<dyn QuestService>>,
) -> impl IntoResponse {
    
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(
        move |socket| handle_socket(socket, token_service, battle_service, quest_service)
    )
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(
    mut socket: WebSocket,
    token_service: Arc<dyn TokenService>, 
    battle_service: Arc<dyn BattleService>,
    quest_service: Arc<dyn QuestService>
) {
    // Send a Request authorization message, and return if error occurs (ie. client disconnects immediately)
    let auth_msg = Message::Text("Auth?".to_string());
    if socket.send(auth_msg).await.is_err() {
        return;
    }

    let user_id = match socket.recv().await {
        Some(Ok(msg)) => Some(token_service.verify_access_token(&msg.into_text().unwrap()).unwrap()),
        _ => None
    };
    if user_id.is_none() {
        socket.send(Message::Text("Invalid access token".to_string())).await.ok();
        return;
    }
    let user_id = user_id.unwrap();

    // Ensure the user is currently on a quest - if not, return BAD REQUEST
    if let Err(e) = quest_service.get_quest(user_id).await {
        if let QuestServiceError::UserNotOnQuest = e {
            socket.send(Message::Text("User does not have an active battle quest".to_string()))
                .await.unwrap();
        }
        return;
    }

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
                tokio::time::sleep(tokio::time::Duration::from_millis(10000)).await;
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