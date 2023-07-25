
pub mod data_layer_error;

pub mod middleware {
    pub mod auth_middleware;
}

pub mod models {
    pub mod game_models;
    pub mod quest_models;
    pub mod auth_models;
}

#[allow(warnings, unused)]
pub mod prisma;

pub mod routes {
    pub mod game_routes;
    pub mod quest_routes;
    pub mod auth_routes;
}

pub mod resources {
    pub mod game_resources;
}

pub mod services {
    pub mod game_service;
    pub mod auth_service;
    pub mod token_service;
    pub mod quest_service;
}