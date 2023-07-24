
pub mod data_layer_error;

pub mod middleware {
    pub mod auth_middleware;
}

pub mod models {
    pub mod game_models;
    pub mod quest_models;
    pub mod auth_models;
}

pub mod prisma;

pub mod routes {
    pub mod game_routes;
    pub mod quest_controller;
    pub mod sabotage_controller;
    pub mod users_controller;
    pub mod auth_routes;
}

pub mod resources {
    pub mod game_resources;
}

pub mod services {
    // pub mod quest_db_services;
    // pub mod user_db_services;
    // pub mod effects_service;
    pub mod game_service;
    pub mod auth_service;
    pub mod token_service;
    pub mod quest_service;
}