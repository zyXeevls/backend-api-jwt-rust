use axum::{Router, routing::post};

use crate::handlers::register_handler::register;

use crate::handlers::login_handler::login;

pub fn auth_routes() -> Router {
    Router::new()
        .route("/api/register", post(register))
        .route("/api/login", post(login))
}
