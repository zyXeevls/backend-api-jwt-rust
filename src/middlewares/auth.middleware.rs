use axum::{
    Json,
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::utils::jwt::verify_token;
use crate::utils::response::ApiResponse;

type AuthError = (StatusCode, Json<ApiResponse<()>>);

pub async fn auth(headers: HeaderMap, mut req: Request, next: Next) -> Result<Response, AuthError> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<()>::error("Missing or invalid token")),
            )
        })?;

    let claims = verify_token(token).map_err(|e| {
        println!("JWT verification error: {:?}", e);
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Invalid token")),
        )
    })?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
