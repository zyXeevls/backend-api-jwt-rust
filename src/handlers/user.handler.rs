use axum::{Extension, Json, extract::Path, http::StatusCode};
use bcrypt::hash;
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use validator::Validate;

use crate::models::user::User;

use crate::utils::response::ApiResponse;

use crate::schemas::user_schema::{UserResponse, UserStoreRequest, UserUpdateRequest};

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

pub async fn update(
    Path(id): Path<i64>,
    Extension(db): Extension<PgPool>,
    Json(payload): Json<UserUpdateRequest>,
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

    if let Some(password) = &payload.password {
        if !password.is_empty() && password.len() < 6 {
            let mut errors = HashMap::new();
            errors.insert(
                "password".to_string(),
                vec!["Password minimal 6 karakter".to_string()],
            );

            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiResponse {
                    status: false,
                    message: "Validasi Gagal".to_string(),
                    data: Some(json!(errors)),
                }),
            );
        }
    }

    let user_exist = match sqlx::query_scalar::<_, i64>("SELECT id FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&db)
        .await
    {
        Ok(Some(user_id)) => user_id,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("User tidak ditemukan")),
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Terjadi kesalahan pada server")),
            );
        }
    };

    let email_exists =
        sqlx::query_scalar::<_, i64>("SELECT id FROM users WHERE email = $1 AND id != $2")
            .bind(&payload.email)
            .bind(user_exist)
            .fetch_optional(&db)
            .await;

    if let Ok(Some(_)) = email_exists {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::error("Email sudah terdaftar")),
        );
    }

    // Update user
    let result = match &payload.password {
        Some(password) if !password.is_empty() => {
            // Hash password Dengan Bcrypt
            let hashed = match hash(password, 10) {
                Ok(h) => h,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::error("Gagal mengenkripsi password")),
                    );
                }
            };

            // Update user dengan password
            sqlx::query("UPDATE users SET name = $1, email = $2, password = $3 WHERE id = $4")
                .bind(&payload.name)
                .bind(&payload.email)
                .bind(&hashed)
                .bind(id)
                .execute(&db)
                .await
        }
        _ => {
            // Update user tanpa password
            sqlx::query("UPDATE users SET name = $1, email = $2 WHERE id = $3")
                .bind(&payload.name)
                .bind(&payload.email)
                .bind(id)
                .execute(&db)
                .await
        }
    };

    if let Err(_) = result {
        return (
            // kirim response 500 Internal Server Error
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("Gagal memperbarui data user")),
        );
    }

    // Ambil data terbaru
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, name, email, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&db)
    .await;

    let user = match user {
        Ok(user) => user,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Gagal mengambil data user terbaru")),
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
        // kirim response 200 OK
        StatusCode::OK,
        Json(ApiResponse::success(
            "User berhasil diperbarui",
            json!(response),
        )),
    )
}
