use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct User {
    pub id: i64,
    pub name: string,
    pub email: string,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
