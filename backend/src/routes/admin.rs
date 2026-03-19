use axum::{extract::{Query, State}, Json};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    auth::AdminUser,
    error::{AppError, Result},
    models::{EventWithCreator, UserPublic},
    routes::AppState,
};

#[derive(Deserialize, ToSchema)]
pub struct IdQuery {
    pub id: String,
}

#[derive(Deserialize, ToSchema)]
pub struct EventIdQuery {
    pub id: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub perms: Option<i16>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub id: String,
    pub username: String,
    pub email: String,
    pub perms: Option<i16>,
    pub new_password: Option<String>,
}

/// List all users (admin)
#[utoipa::path(
    get,
    path = "/api/admin/users",
    tag = "Admin",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "All users", body = Vec<UserPublic>),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_users(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<UserPublic>>> {
    let users = sqlx::query_as::<_, UserPublic>(
        r#"SELECT id, username, email, perms,
           "createdAt" AT TIME ZONE 'UTC' AS created_at
           FROM "Users" ORDER BY "createdAt" ASC"#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(users))
}

/// Create a new user (admin)
#[utoipa::path(
    post,
    path = "/api/admin/users",
    tag = "Admin",
    security(("bearer_auth" = [])),
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "Created user", body = UserPublic),
        (status = 409, description = "Email already in use"),
    )
)]
pub async fn create_user(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> Result<Json<UserPublic>> {
    let username = body.username.trim().to_string();
    let email = body.email.trim().to_string();

    if username.is_empty() || email.is_empty() || body.password.is_empty() {
        return Err(AppError::BadRequest("Username, email and password are required".into()));
    }
    if body.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
    }

    let exists = sqlx::query(r#"SELECT id FROM "Users" WHERE email = $1"#)
        .bind(&email)
        .fetch_optional(&state.db)
        .await?
        .is_some();
    if exists { return Err(AppError::Conflict("Email already in use".into())); }

    let hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let new_id = uuid::Uuid::new_v4().to_string();
    let perms = body.perms.unwrap_or(0);

    let user = sqlx::query_as::<_, UserPublic>(
        r#"INSERT INTO "Users" (id, username, email, "passwordHash", perms)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, username, email, perms,
           "createdAt" AT TIME ZONE 'UTC' AS created_at"#
    )
    .bind(&new_id)
    .bind(&username)
    .bind(&email)
    .bind(&hash)
    .bind(perms)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(user))
}

/// Update a user (admin)
#[utoipa::path(
    put,
    path = "/api/admin/users",
    tag = "Admin",
    security(("bearer_auth" = [])),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "Updated user", body = UserPublic),
        (status = 404, description = "User not found"),
    )
)]
pub async fn update_user(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<Json<UserPublic>> {
    let username = body.username.trim().to_string();
    let email = body.email.trim().to_string();
    let perms = body.perms.unwrap_or(0);

    if username.is_empty() || email.is_empty() {
        return Err(AppError::BadRequest("Username and email are required".into()));
    }

    let updated = if let Some(new_pw) = &body.new_password {
        if new_pw.len() < 8 {
            return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
        }
        let hash = bcrypt::hash(new_pw, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        sqlx::query_as::<_, UserPublic>(
            r#"UPDATE "Users" SET username = $1, email = $2, perms = $3, "passwordHash" = $4
               WHERE id = $5
               RETURNING id, username, email, perms,
               "createdAt" AT TIME ZONE 'UTC' AS created_at"#
        )
        .bind(&username)
        .bind(&email)
        .bind(perms)
        .bind(&hash)
        .bind(&body.id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?
    } else {
        sqlx::query_as::<_, UserPublic>(
            r#"UPDATE "Users" SET username = $1, email = $2, perms = $3
               WHERE id = $4
               RETURNING id, username, email, perms,
               "createdAt" AT TIME ZONE 'UTC' AS created_at"#
        )
        .bind(&username)
        .bind(&email)
        .bind(perms)
        .bind(&body.id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?
    };

    Ok(Json(updated))
}

/// Delete a user (admin)
#[utoipa::path(
    delete,
    path = "/api/admin/users",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = String, Query, description = "User ID")),
    responses(
        (status = 200, description = "Deleted"),
        (status = 404, description = "User not found"),
    )
)]
pub async fn delete_user(
    admin: AdminUser,
    State(state): State<AppState>,
    Query(q): Query<IdQuery>,
) -> Result<Json<serde_json::Value>> {
    if q.id == admin.0.sub {
        return Err(AppError::BadRequest("Cannot delete your own account".into()));
    }

    let deleted = sqlx::query(
        r#"DELETE FROM "Users" WHERE id = $1 RETURNING id"#
    )
    .bind(&q.id)
    .fetch_optional(&state.db)
    .await?
    .is_some();

    if !deleted { return Err(AppError::NotFound); }
    Ok(Json(serde_json::json!({ "success": true })))
}

/// List all events (admin)
#[utoipa::path(
    get,
    path = "/api/admin/events",
    tag = "Admin",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "All events", body = Vec<EventWithCreator>),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_events(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<EventWithCreator>>> {
    let events = sqlx::query_as::<_, EventWithCreator>(
        r#"
        SELECT e.id, e.title, e.description, e.date AT TIME ZONE 'UTC' AS date,
               e.location, e.color, e."createdBy" AS created_by,
               e."createdAt" AT TIME ZONE 'UTC' AS created_at,
               e.private, u.username AS creator_name
        FROM "Events" e
        LEFT JOIN "Users" u ON e."createdBy" = u.id
        ORDER BY e.date ASC
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(events))
}

/// Delete an event (admin)
#[utoipa::path(
    delete,
    path = "/api/admin/events",
    tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = i64, Query, description = "Event ID")),
    responses(
        (status = 200, description = "Deleted"),
        (status = 404, description = "Event not found"),
    )
)]
pub async fn delete_event(
    _admin: AdminUser,
    State(state): State<AppState>,
    Query(q): Query<EventIdQuery>,
) -> Result<Json<serde_json::Value>> {
    let deleted = sqlx::query(
        r#"DELETE FROM "Events" WHERE id = $1 RETURNING id"#
    )
    .bind(q.id)
    .fetch_optional(&state.db)
    .await?
    .is_some();

    if !deleted { return Err(AppError::NotFound); }
    Ok(Json(serde_json::json!({ "success": true })))
}
