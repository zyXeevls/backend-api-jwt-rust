use axum::{Extension, Json, http::StatusCode};
use bcrypt::hash;
use serde_json::{Value, json};
use sqlx::MySqlPool;
use std::collections::HashMap;
use validator::Validate;

// import schema request dan response register
use crate::schemas::register_schema::{RegisterRequest, RegisterResponse};

// import util response API
use crate::utils::response::ApiResponse;

pub async fn register(
    Extension(db): Extension<MySqlPool>,
    Json(payload): Json<RegisterRequest>,
) -> (StatusCode, Json<ApiResponse<Value>>) {
    // Validasi Request
    if let Err(errors) = payload.validate() {
        let mut field_errors: HashMap<String, Vec<String>> = HashMap::new();

        // kumpulkan semua error dari validasi
        for (field, errors) in errors.field_errors() {
            let messages = errors
                .iter()
                .filter_map(|e| e.message.as_ref())
                .map(|m| m.to_string())
                .collect::<Vec<String>>();

            field_errors.insert(field.to_string(), messages);
        }

        return (
            // kirim response 422 Unprocessable Entity
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse {
                status: false,
                message: "Validasi Gagal".to_string(),
                data: Some(json!(field_errors)),
            }),
        );
    }

    // Hash Password Dengan Bcrypt
    let password = match hash(payload.password, 10) {
        Ok(hashed) => hashed,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Gagal mengenkripsi password")),
            );
        }
    };

    // Insert Data User ke Database
    let result = sqlx::query!(
        "INSERT INTO users (name, email, password) VALUES (?, ?, ?)",
        payload.name,
        payload.email,
        password
    )
    .execute(&db)
    .await;

    match result {
        Ok(result) => {
            // get id user yang baru saja dibuat
            let user_id = result.last_insert_id() as i64;

            // Ambil data user berdasarkan ID
            let user = sqlx::query!(
                r#"
                SELECT id, name, email, created_at, updated_at
                FROM users
                WHERE id = ?
                "#,
                user_id
            )
            .fetch_one(&db)
            .await;

            match user {
                Ok(user) => {
                    let response = RegisterResponse {
                        id: user.id,
                        name: user.name,
                        email: user.email,
                        created_at: user.created_at,
                        updated_at: user.updated_at,
                    };

                    (
                        // kirim response 201 Created
                        StatusCode::CREATED,
                        Json(ApiResponse::success("Register Berhasil!", json!(response))),
                    )
                }
                Err(_) => (
                    // kirim response 500 Internal Server Error
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error("Gagal mengambil data user")),
                ),
            }
        }
        Err(e) => {
            if e.to_string().contains("Duplicate entry") {
                (
                    // kirim response 409 Conflict
                    StatusCode::CONFLICT,
                    Json(ApiResponse::error("Email sudah terdaftar")),
                )
            } else {
                (
                    // kirim response 500 Internal Server Error
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error("Register Gagal!")),
                )
            }
        }
    }
}
