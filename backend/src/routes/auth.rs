use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{generate_access_token, generate_refresh_token, store_refresh_token, validate_refresh_token, revoke_refresh_token, AuthUser},
    error::{AppError, Result},
    models::{User, UserPublic},
    routes::AppState,
};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserPublic,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    let user = sqlx::query_as::<_, User>(
        r#"SELECT id, username, email, "passwordHash" AS password_hash, perms, "createdAt" AS created_at FROM "Users" WHERE email = $1"#
    )
    .bind(&body.email)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let valid = bcrypt::verify(&body.password, &user.password_hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    if !valid {
        return Err(AppError::Unauthorized);
    }

    let access_token = generate_access_token(&user.id, &user.email, user.perms, &state.cfg)?;
    let refresh_token = generate_refresh_token();
    store_refresh_token(&state.redis, &refresh_token, &user.id, state.cfg.refresh_token_expiry_secs).await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user: UserPublic {
            id: user.id,
            username: user.username,
            email: user.email,
            perms: user.perms,
            created_at: user.created_at,
        },
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(body): Json<LogoutRequest>,
) -> Result<Json<serde_json::Value>> {
    revoke_refresh_token(&state.redis, &body.refresh_token).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>> {
    let user_id = validate_refresh_token(&state.redis, &body.refresh_token).await?;

    let user = sqlx::query_as::<_, User>(
        r#"SELECT id, username, email, "passwordHash" AS password_hash, perms, "createdAt" AS created_at FROM "Users" WHERE id = $1"#
    )
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let access_token = generate_access_token(&user.id, &user.email, user.perms, &state.cfg)?;
    Ok(Json(serde_json::json!({ "access_token": access_token })))
}

pub async fn me(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<UserPublic>> {
    let user = sqlx::query_as::<_, UserPublic>(
        r#"SELECT id, username, email, perms, "createdAt" AS created_at FROM "Users" WHERE id = $1"#
    )
    .bind(&auth.0.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(user))
}
