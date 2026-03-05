use axum::{Extension, Json, http::StatusCode};

use bcrypt::verify;
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use validator::Validate;

use crate::schemas::login_schema::{LoginRequest, LoginResponse, UserResponse};

use crate::utils::{jwt::generate_token, response::ApiResponse};

#[derive(Debug, FromRow)]
struct LoginUserRow {
    id: i64,
    name: String,
    email: String,
    password: String,
}

pub async fn login(
    Extension(db): Extension<PgPool>,
    Json(payload): Json<LoginRequest>,
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
                message: "Validation failed".to_string(),
                data: Some(json!({ "errors": field_errors })),
            }),
        );
    }

    let user = match sqlx::query_as::<_, LoginUserRow>(
        "SELECT id, name, email, password FROM users WHERE email = $1",
    )
    .bind(&payload.email)
    .fetch_optional(&db)
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::error("Email atau Password anda salah")),
            );
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Terjadi kesalahan pada server")),
            );
        }
    };

    match verify(&payload.password, &user.password) {
        Ok(true) => match generate_token(user.id) {
            Ok(token) => {
                let response = LoginResponse {
                    user: UserResponse {
                        id: user.id,
                        name: user.name,
                        email: user.email,
                    },
                    token,
                };

                (
                    StatusCode::OK,
                    Json(ApiResponse::success("Login berhasil", json!(response))),
                )
            }
            Err(e) => {
                eprintln!("JWT generation error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error("Terjadi kesalahan pada server")),
                )
            }
        },
        Ok(false) => (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error("Email atau password anda salah")),
        ),

        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("Gagal memverifikasi password")),
        ),
    }
}
