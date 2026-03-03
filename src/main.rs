use axum::{Extension, Router};
use dotenvy::dotenv;
use std::net::SocketAddr;

mod config;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db = config::database::connect().await;

    let app = Router::new().layer(Extension(db));

    let port = std::env::var("APP_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(3001);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    println!("Server running at http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
