use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::handlers::user_handler::{index, show, store};

use crate::middlewares::auth_middleware::auth;

pub fn user_routes() -> Router {
    Router::new()
        .route("/api/users", get(index))
        .route("/api/users", post(store))
        .route("/api/users/{id}", get(show))
        .layer(middleware::from_fn(auth))
}
