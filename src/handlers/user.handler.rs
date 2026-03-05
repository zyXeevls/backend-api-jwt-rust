use axum::{Extension, Json, extract::Path, http::StatusCode};
use bcrypt::hash;
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use validator::Validate;

use crate::models::user::User;

use crate::utils::response::ApiResponse;

use crate::schemas::user_schema::{UserResponse, UserStoreRequest};

#[derive(Debug, FromRow)]
struct UserStoreRow {
    id: i64,
    name: String,
    email: String,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

pub async fn index(Extension(db): Extension<PgPool>) -> (StatusCode, Json<ApiResponse<Value>>) {
    let users = match sqlx::query_as::<_, User>(
        r#"
        SELECT id, name, email, created_at, updated_at
        FROM users
        ORDER BY id DESC
        "#,
    )
    .fetch_all(&db)
    .await
    {
        Ok(users) => users,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Gagal mengambil data user")),
            );
        }
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success("List user", json!(users))),
    )
}

pub async fn store(
    Extension(db): Extension<PgPool>,
    Json(payload): Json<UserStoreRequest>,
) -> (StatusCode, Json<ApiResponse<Value>>) {
    if let Err(errors) = payload.validate() {
        let mut field_errors: HashMap<String, Vec<String>> = HashMap::new();

        for (field, field_errors_data) in errors.field_errors() {
            let messages = field_errors_data
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

    let result = sqlx::query_as::<_, UserStoreRow>(
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
            let response = UserResponse {
                id: user.id,
                name: user.name,
                email: user.email,
                created_at: user.created_at,
                updated_at: user.updated_at,
            };

            (
                StatusCode::CREATED,
                Json(ApiResponse::success(
                    "User berhasil ditambahkan",
                    json!(response),
                )),
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
                Json(ApiResponse::error("Gagal menambahkan user")),
            )
        }
    }
}

pub async fn show(
    Path(id): Path<i64>,
    Extension(db): Extension<PgPool>,
) -> (StatusCode, Json<ApiResponse<Value>>) {
    let user = match sqlx::query_as::<_, User>(
        r#"
        SELECT id, name, email, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&db)
    .await
    {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("User tidak ditemukan")),
            );
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Gagal mengambil data user")),
            );
        }
    };

    let response = UserResponse {
        id: user.id,
        name: user.name,
        email: user.email,
        created_at: user.created_at,
        updated_at: user.updated_at,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success("Detail user", json!(response))),
    )
}
