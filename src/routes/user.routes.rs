use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};

use crate::handlers::user_handler::{destroy, index, show, store, update};

use crate::middlewares::auth_middleware::auth;

pub fn user_routes() -> Router {
    Router::new()
        .route("/api/users", get(index))
        .route("/api/users", post(store))
        .route("/api/users/{id}", get(show))
        .route("/api/users/{id}", put(update))
        .route("/api/users/{id}", delete(destroy))
        .layer(middleware::from_fn(auth))
}
