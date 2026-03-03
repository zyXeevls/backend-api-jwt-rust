use chrono::{Duration, Utc};
use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Error as JwtError,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Claims {
    pub sub: i64,
    pub exp: usize,
}

pub fn generate_token(user_id: i64) -> Result<String, JwtError> {
    let exp = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    encode(
        &Header::default(),
        &Claims { sub: user_id, exp },
        &EncodingKey::from_secret(
            std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "secret".to_string())
                .as_ref(),
        ),
    )
}

pub fn verify_token(token: &str) -> Result<Claims, JwtError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(
            std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "secret".to_string())
                .as_ref(),
        ),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
