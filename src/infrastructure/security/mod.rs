use anyhow::Context;
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
    .context(format!("Couldn't encode a token for this sub {} ", sub))
}

pub fn verify_token(token: &str, secret_key: &[u8]) -> anyhow::Result<uuid::Uuid> {
    let token = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret_key),
        &Validation::default(),
    )?;
    Ok(token.claims.sub)
}

pub fn hash_password(pwd: &[u8], salt: &[u8]) -> anyhow::Result<String> {
    argon2::hash_encoded(pwd, salt, &argon2::Config::default())
        .context("Couldn't hash this password")
}

pub fn verify_password(pwd: &[u8], hash: &str) -> anyhow::Result<bool> {
    argon2::verify_encoded(hash, pwd).context("Couldn't verify this password")
}

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
        let verified_sub = verify_token(&token[..], b"mysupersecretkey").unwrap();

        assert_eq!(sub, verified_sub);
    }

    #[test]
    fn should_hash_a_password_correctly() {
        let pwd = "453cR37";
        let hash = hash_password(pwd.as_bytes(), b"randomsalt").unwrap();
        assert!(verify_password(pwd.as_bytes(), &hash[..]).unwrap())
    }
}
