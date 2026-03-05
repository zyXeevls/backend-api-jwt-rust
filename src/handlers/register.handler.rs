use axum::{Extension, Json, http::StatusCode};
use bcrypt::hash;
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use validator::Validate;

use crate::schemas::register_schema::{RegisterRequest, RegisterResponse};
use crate::utils::response::ApiResponse;

#[derive(Debug, FromRow)]
struct RegisterUserRow {
    id: i64,
    name: String,
    email: String,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

pub async fn register(
    Extension(db): Extension<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> (StatusCode, Json<ApiResponse<Value>>) {
    if let Err(errors) = payload.validate() {
        let mut field_errors: HashMap<String, Vec<String>> = HashMap::new();

        for (field, errors) in errors.field_errors() {
            let messages = errors
                .iter()
                .filter_map(|e| e.message.as_ref())
                .map(|m| m.to_string())
                .collect::<Vec<String>>();

            field_errors.insert(field.to_string(), messages);
        }

        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse {
                status: false,
                message: "Validasi Gagal".to_string(),
                data: Some(json!(field_errors)),
            }),
        );
    }

    let hashed_password = match hash(payload.password, 10) {
        Ok(hashed) => hashed,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Gagal mengenkripsi password")),
            );
        }
    };

    let result = sqlx::query_as::<_, RegisterUserRow>(
        r#"
        INSERT INTO users (name, email, password)
        VALUES ($1, $2, $3)
        RETURNING id, name, email, created_at, updated_at
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.email)
    .bind(&hashed_password)
    .fetch_one(&db)
    .await;

    match result {
        Ok(user) => {
            let response = RegisterResponse {
                id: user.id,
                name: user.name,
                email: user.email,
                created_at: user.created_at,
                updated_at: user.updated_at,
            };

            (
                StatusCode::CREATED,
                Json(ApiResponse::success("Register Berhasil!", json!(response))),
            )
        }
        Err(e) => {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().as_deref() == Some("23505") {
                    return (
                        StatusCode::CONFLICT,
                        Json(ApiResponse::error("Email sudah terdaftar")),
                    );
                }
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Register Gagal!")),
            )
        }
    }
}
