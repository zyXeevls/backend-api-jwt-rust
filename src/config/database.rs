use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn connect() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    match PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("Connected to the database successfully.");
            pool
        }
        Err(err) => {
            eprintln!("Failed to connect to the database: {}", err);
            std::process::exit(1);
        }
    }
}
