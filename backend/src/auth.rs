use crate::{config::Config, error::{AppError, Result}, models::Claims};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use chrono::Utc;
use uuid::Uuid;
use redis::AsyncCommands;

// ── Token generation ──────────────────────────────────────────────────────────

pub fn generate_access_token(user_id: &str, email: &str, perms: i16, cfg: &Config) -> Result<String> {
    let now = Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        perms,
        iat: now,
        exp: now + cfg.access_token_expiry_secs as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))
}

pub fn generate_refresh_token() -> String {
    Uuid::new_v4().to_string()
}

pub async fn store_refresh_token(
    redis: &redis::Client,
    token: &str,
    user_id: &str,
    expiry_secs: u64,
) -> Result<()> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    let key = format!("refresh:{}", token);
    conn.set_ex::<_, _, ()>(key, user_id, expiry_secs).await?;
    Ok(())
}

pub async fn validate_refresh_token(
    redis: &redis::Client,
    token: &str,
) -> Result<String> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    let key = format!("refresh:{}", token);
    let user_id: Option<String> = conn.get(&key).await?;
    user_id.ok_or(AppError::Unauthorized)
}

pub async fn revoke_refresh_token(redis: &redis::Client, token: &str) -> Result<()> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    let key = format!("refresh:{}", token);
    conn.del::<_, ()>(key).await?;
    Ok(())
}

// ── JWT validation ────────────────────────────────────────────────────────────

pub fn verify_access_token(token: &str, secret: &str) -> Result<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}

// ── Axum extractor for authenticated user ─────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AuthUser(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    crate::routes::AppState: axum::extract::FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> std::result::Result<Self, AppError> {
        use axum::extract::FromRef;
        let app_state = crate::routes::AppState::from_ref(state);

        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let claims = verify_access_token(token, &app_state.cfg.jwt_secret)?;
        Ok(AuthUser(claims))
    }
}

#[derive(Debug, Clone)]
pub struct AdminUser(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
    crate::routes::AppState: axum::extract::FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> std::result::Result<Self, AppError> {
        let AuthUser(claims) = AuthUser::from_request_parts(parts, state).await?;
        if claims.perms < 999 {
            return Err(AppError::Forbidden);
        }
        Ok(AdminUser(claims))
    }
}
