use axum::{extract::State, Json};
use serde::Deserialize;

use crate::{
    auth::AuthUser,
    error::{AppError, Result},
    models::UserPublic,
    routes::AppState,
};

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    pub username: String,
    pub email: String,
    pub current_password: Option<String>,
    pub new_password: Option<String>,
}

// GET /api/account
pub async fn get_profile(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<UserPublic>> {
    let user = sqlx::query_as!(
        UserPublic,
        r#"SELECT id, username, email, perms, "createdAt" AS created_at FROM "Users" WHERE id = $1"#,
        auth.0.sub
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(user))
}

// PUT /api/account
pub async fn update_profile(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<Json<UserPublic>> {
    let username = body.username.trim().to_string();
    let email = body.email.trim().to_string();

    if username.is_empty() || email.is_empty() {
        return Err(AppError::BadRequest("Username and email are required".into()));
    }

    // Check username conflict
    let taken = sqlx::query!(
        r#"SELECT id FROM "Users" WHERE username = $1 AND id != $2"#,
        username, auth.0.sub
    )
    .fetch_optional(&state.db)
    .await?
    .is_some();
    if taken { return Err(AppError::Conflict("Username already taken".into())); }

    // Check email conflict
    let email_taken = sqlx::query!(
        r#"SELECT id FROM "Users" WHERE email = $1 AND id != $2"#,
        email, auth.0.sub
    )
    .fetch_optional(&state.db)
    .await?
    .is_some();
    if email_taken { return Err(AppError::Conflict("Email already in use".into())); }

    // Handle password change
    let new_hash: Option<String> = if let Some(new_pw) = &body.new_password {
        let current_pw = body.current_password.as_deref()
            .ok_or_else(|| AppError::BadRequest("Current password required".into()))?;

        if new_pw.len() < 8 {
            return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
        }

        let user = sqlx::query!(
            r#"SELECT "passwordHash" AS password_hash FROM "Users" WHERE id = $1"#,
            auth.0.sub
        )
        .fetch_one(&state.db)
        .await?;

        let valid = bcrypt::verify(current_pw, &user.password_hash)
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        if !valid {
            return Err(AppError::BadRequest("Current password is incorrect".into()));
        }

        Some(bcrypt::hash(new_pw, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?)
    } else {
        None
    };

    // Update user
    let updated = if let Some(hash) = new_hash {
        sqlx::query_as!(
            UserPublic,
            r#"UPDATE "Users" SET username = $1, email = $2, "passwordHash" = $3
               WHERE id = $4
               RETURNING id, username, email, perms, "createdAt" AS created_at"#,
            username, email, hash, auth.0.sub
        )
        .fetch_one(&state.db)
        .await?
    } else {
        sqlx::query_as!(
            UserPublic,
            r#"UPDATE "Users" SET username = $1, email = $2
               WHERE id = $3
               RETURNING id, username, email, perms, "createdAt" AS created_at"#,
            username, email, auth.0.sub
        )
        .fetch_one(&state.db)
        .await?
    };

    Ok(Json(updated))
}
