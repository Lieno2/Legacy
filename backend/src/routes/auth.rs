use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    auth::{generate_access_token, generate_refresh_token, store_refresh_token, validate_refresh_token, revoke_refresh_token, AuthUser},
    error::{AppError, Result},
    models::{User, UserPublic},
    routes::AppState,
};

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserPublic,
}

#[derive(Deserialize, ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

/// Login with email and password
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 429, description = "Too many requests"),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    tracing::debug!("[LOGIN] Attempt for email: {}", body.email);

    // Fetch user including avatar_url so it is available in the response
    let user = sqlx::query_as::<_, User>(
        r#"SELECT id, username, email, "passwordHash" AS password_hash, perms,
           "createdAt" AT TIME ZONE 'UTC' AS created_at
           FROM "Users" WHERE email = $1"#
    )
    .bind(&body.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| { tracing::error!("[LOGIN] DB error fetching user: {}", e); e })?;

    let user = match user {
        Some(u) => {
            tracing::debug!("[LOGIN] User found: id={} perms={}", u.id, u.perms);
            u
        }
        None => {
            tracing::warn!("[LOGIN] No user found for email: {}", body.email);
            return Err(AppError::Unauthorized);
        }
    };

    let valid = bcrypt::verify(&body.password, &user.password_hash)
        .map_err(|e| { tracing::error!("[LOGIN] bcrypt error: {}", e); AppError::Internal(anyhow::anyhow!(e)) })?;

    if !valid {
        tracing::warn!("[LOGIN] Wrong password for email: {}", body.email);
        return Err(AppError::Unauthorized);
    }

    tracing::debug!("[LOGIN] Password valid, generating tokens");

    let access_token = generate_access_token(&user.id, &user.email, user.perms, &state.cfg)?;
    let refresh_token = generate_refresh_token();
    store_refresh_token(&state.redis, &refresh_token, &user.id, state.cfg.refresh_token_expiry_secs).await
        .map_err(|e| { tracing::error!("[LOGIN] Redis error storing refresh token: {}", e); e })?;

    // Fetch avatar_url separately (not on User model to avoid schema changes)
    let avatar_url = sqlx::query_scalar::<_, Option<String>>(
        r#"SELECT avatar_url FROM "Users" WHERE id = $1"#
    )
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None)
    .flatten();

    tracing::info!("[LOGIN] \u2705 Success for email: {}", body.email);

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user: UserPublic {
            id: user.id,
            username: user.username,
            email: user.email,
            perms: user.perms,
            created_at: user.created_at,
            avatar_url,
        },
    }))
}

/// Logout and revoke refresh token
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "Auth",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Logged out"),
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    Json(body): Json<LogoutRequest>,
) -> Result<Json<serde_json::Value>> {
    revoke_refresh_token(&state.redis, &body.refresh_token).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

/// Refresh access token
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "Auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "New access token issued"),
        (status = 401, description = "Invalid or expired refresh token"),
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>> {
    let user_id = validate_refresh_token(&state.redis, &body.refresh_token).await?;

    let user = sqlx::query_as::<_, User>(
        r#"SELECT id, username, email, "passwordHash" AS password_hash, perms,
           "createdAt" AT TIME ZONE 'UTC' AS created_at
           FROM "Users" WHERE id = $1"#
    )
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let access_token = generate_access_token(&user.id, &user.email, user.perms, &state.cfg)?;
    Ok(Json(serde_json::json!({ "access_token": access_token })))
}

/// Get current authenticated user
#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "Auth",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Current user", body = UserPublic),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn me(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<UserPublic>> {
    let user = sqlx::query_as::<_, UserPublic>(
        r#"SELECT id, username, email, perms, avatar_url,
           "createdAt" AT TIME ZONE 'UTC' AS created_at
           FROM "Users" WHERE id = $1"#
    )
    .bind(&auth.0.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(user))
}
