use chrono::serde::ts_seconds;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

// FIXME: Keep the config to avoid repeating it in the methods
pub fn sign_token(
    sub: uuid::Uuid,
    expiration_time: i64,
    secret_key: &[u8],
) -> anyhow::Result<String> {
    let exp = chrono::Utc::now() + chrono::Duration::seconds(expiration_time);
    let claims = Claims { sub, exp };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key),
    )
    .map_err(anyhow::Error::new)
}

#[allow(dead_code)] //FIXME
pub fn verify_token(token: &str, secret_key: &[u8]) -> anyhow::Result<uuid::Uuid> {
    let token = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret_key),
        &Validation::default(),
    )?;
    Ok(token.claims.sub)
}

// pub fn hash_password(password: &[u8]) -> anyhow::Result<String> {
//     argon2::hash_encoded(
//         password,
//         &self.config.hash_salt(),
//         &argon2::Config::default(),
//     )
//     .map_err(anyhow::Error::new)
// }

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: uuid::Uuid,
    #[serde(with = "ts_seconds")]
    exp: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_a_valid_token() {
        let sub = uuid::Uuid::new_v4();
        let token = sign_token(sub, 3600, b"mysupersecretkey").unwrap();
        let sub_verified = verify_token(&token[..], b"mysupersecretkey").unwrap();

        assert_eq!(sub, sub_verified);
    }
}
