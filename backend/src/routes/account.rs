use axum::{extract::State, Json};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    auth::AuthUser,
    error::{AppError, Result},
    models::UserPublic,
    routes::AppState,
};

#[derive(Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    pub username: String,
    pub email: String,
    pub current_password: Option<String>,
    pub new_password: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(sqlx::FromRow)]
struct PasswordRow {
    password_hash: String,
}

/// Get current user profile
#[utoipa::path(
    get, path = "/api/account", tag = "Account",
    security(("bearer_auth" = [])),
    responses((status = 200, body = UserPublic), (status = 401))
)]
pub async fn get_profile(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<UserPublic>> {
    let user = sqlx::query_as::<_, UserPublic>(
        r#"SELECT id, username, email, perms,
           "createdAt" AT TIME ZONE 'UTC' AS created_at,
           avatar_url
           FROM "Users" WHERE id = $1"#
    )
    .bind(&auth.0.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(user))
}

/// Update current user profile
#[utoipa::path(
    put, path = "/api/account", tag = "Account",
    security(("bearer_auth" = [])),
    request_body = UpdateProfileRequest,
    responses((status = 200, body = UserPublic), (status = 400), (status = 409))
)]
pub async fn update_profile(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<Json<UserPublic>> {
    let username = body.username.trim().to_string();
    let email    = body.email.trim().to_string();

    if username.is_empty() || email.is_empty() {
        return Err(AppError::BadRequest("Username and email are required".into()));
    }

    // Validate avatar size — base64 of 128x128 JPEG is ~10-15KB, cap at 200KB
    if let Some(ref av) = body.avatar_url {
        if av.len() > 200_000 {
            return Err(AppError::BadRequest("Avatar image is too large (max 200KB)".into()));
        }
    }

    let taken = sqlx::query(r#"SELECT id FROM "Users" WHERE username = $1 AND id != $2"#)
        .bind(&username).bind(&auth.0.sub)
        .fetch_optional(&state.db).await?.is_some();
    if taken { return Err(AppError::Conflict("Username already taken".into())); }

    let email_taken = sqlx::query(r#"SELECT id FROM "Users" WHERE email = $1 AND id != $2"#)
        .bind(&email).bind(&auth.0.sub)
        .fetch_optional(&state.db).await?.is_some();
    if email_taken { return Err(AppError::Conflict("Email already in use".into())); }

    let new_hash: Option<String> = if let Some(new_pw) = &body.new_password {
        let current_pw = body.current_password.as_deref()
            .ok_or_else(|| AppError::BadRequest("Current password required".into()))?;

        if new_pw.len() < 8 {
            return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
        }

        let row = sqlx::query_as::<_, PasswordRow>(
            r#"SELECT "passwordHash" AS password_hash FROM "Users" WHERE id = $1"#
        )
        .bind(&auth.0.sub).fetch_one(&state.db).await?;

        let valid = bcrypt::verify(current_pw, &row.password_hash)
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        if !valid { return Err(AppError::BadRequest("Current password is incorrect".into())); }

        Some(bcrypt::hash(new_pw, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?)
    } else {
        None
    };

    let updated = if let Some(hash) = new_hash {
        sqlx::query_as::<_, UserPublic>(
            r#"UPDATE "Users" SET username=$1, email=$2, "passwordHash"=$3, avatar_url=$4
               WHERE id=$5
               RETURNING id, username, email, perms,
               "createdAt" AT TIME ZONE 'UTC' AS created_at, avatar_url"#
        )
        .bind(&username).bind(&email).bind(&hash).bind(&body.avatar_url).bind(&auth.0.sub)
        .fetch_one(&state.db).await?
    } else {
        sqlx::query_as::<_, UserPublic>(
            r#"UPDATE "Users" SET username=$1, email=$2, avatar_url=$3
               WHERE id=$4
               RETURNING id, username, email, perms,
               "createdAt" AT TIME ZONE 'UTC' AS created_at, avatar_url"#
        )
        .bind(&username).bind(&email).bind(&body.avatar_url).bind(&auth.0.sub)
        .fetch_one(&state.db).await?
    };

    Ok(Json(updated))
}