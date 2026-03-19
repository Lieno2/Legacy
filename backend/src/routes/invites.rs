use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    auth::AuthUser,
    error::{AppError, Result},
    routes::AppState,
};

#[derive(Deserialize, ToSchema)]
pub struct EventIdQuery {
    pub event_id: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct SearchQuery {
    pub q: String,
    pub event_id: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct InviteRequest {
    pub event_id: i64,
    pub user_id: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RemoveInviteRequest {
    pub event_id: i64,
    pub user_id: String,
}

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct InviteUser {
    pub id: String,
    pub username: String,
    pub email: String,
}

/// Search users to invite (excludes self and already-invited)
#[utoipa::path(
    get,
    path = "/api/invites/search",
    tag = "Invites",
    security(("bearer_auth" = [])),
    params(
        ("q" = String, Query, description = "Search query"),
        ("event_id" = i64, Query, description = "Event ID"),
    ),
    responses(
        (status = 200, description = "Matching users", body = Vec<InviteUser>),
    )
)]
pub async fn search_users(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<InviteUser>>> {
    let pattern = format!("%{}%", q.q.to_lowercase());
    let users = sqlx::query_as::<_, InviteUser>(
        r#"
        SELECT id, username, email FROM "Users"
        WHERE id != $1
          AND id NOT IN (
              SELECT "userId" FROM "EventMembers" WHERE "eventId" = $2
          )
          AND (LOWER(username) LIKE $3 OR LOWER(email) LIKE $3)
        ORDER BY username
        LIMIT 10
        "#
    )
    .bind(&auth.0.sub)
    .bind(q.event_id)
    .bind(&pattern)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(users))
}

/// List invited users for a private event
#[utoipa::path(
    get,
    path = "/api/invites",
    tag = "Invites",
    security(("bearer_auth" = [])),
    params(("event_id" = i64, Query, description = "Event ID")),
    responses(
        (status = 200, description = "Invited users", body = Vec<InviteUser>),
    )
)]
pub async fn list(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<EventIdQuery>,
) -> Result<Json<Vec<InviteUser>>> {
    // Only owner can see invites
    let is_owner = sqlx::query_scalar::<_, bool>(
        r#"SELECT EXISTS(SELECT 1 FROM "Events" WHERE id = $1 AND "createdBy" = $2)"#
    )
    .bind(q.event_id)
    .bind(&auth.0.sub)
    .fetch_one(&state.db)
    .await?;

    if !is_owner {
        return Err(AppError::Forbidden);
    }

    let users = sqlx::query_as::<_, InviteUser>(
        r#"
        SELECT u.id, u.username, u.email
        FROM "EventMembers" em
        JOIN "Users" u ON em."userId" = u.id
        WHERE em."eventId" = $1
        ORDER BY u.username
        "#
    )
    .bind(q.event_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(users))
}

/// Invite a user to a private event (inserts into EventMembers with status 'going' as placeholder)
#[utoipa::path(
    post,
    path = "/api/invites",
    tag = "Invites",
    security(("bearer_auth" = [])),
    request_body = InviteRequest,
    responses(
        (status = 200, description = "User invited"),
        (status = 403, description = "Not event owner"),
        (status = 404, description = "Event or user not found"),
    )
)]
pub async fn invite(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<InviteRequest>,
) -> Result<Json<serde_json::Value>> {
    let is_owner = sqlx::query_scalar::<_, bool>(
        r#"SELECT EXISTS(SELECT 1 FROM "Events" WHERE id = $1 AND "createdBy" = $2 AND private = true)"#
    )
    .bind(body.event_id)
    .bind(&auth.0.sub)
    .fetch_one(&state.db)
    .await?;

    if !is_owner {
        return Err(AppError::Forbidden);
    }

    let user_exists = sqlx::query_scalar::<_, bool>(
        r#"SELECT EXISTS(SELECT 1 FROM "Users" WHERE id = $1)"#
    )
    .bind(&body.user_id)
    .fetch_one(&state.db)
    .await?;

    if !user_exists {
        return Err(AppError::NotFound);
    }

    // Insert into EventMembers — no status yet (invited but not RSVPed)
    sqlx::query(
        r#"
        INSERT INTO "EventMembers" ("eventId", "userId", status)
        VALUES ($1, $2, 'invited')
        ON CONFLICT ("eventId", "userId") DO NOTHING
        "#
    )
    .bind(body.event_id)
    .bind(&body.user_id)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Remove a user from a private event and clear their RSVP
#[utoipa::path(
    delete,
    path = "/api/invites",
    tag = "Invites",
    security(("bearer_auth" = [])),
    request_body = RemoveInviteRequest,
    responses(
        (status = 200, description = "User removed"),
        (status = 403, description = "Not event owner"),
    )
)]
pub async fn remove(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<RemoveInviteRequest>,
) -> Result<Json<serde_json::Value>> {
    let is_owner = sqlx::query_scalar::<_, bool>(
        r#"SELECT EXISTS(SELECT 1 FROM "Events" WHERE id = $1 AND "createdBy" = $2)"#
    )
    .bind(body.event_id)
    .bind(&auth.0.sub)
    .fetch_one(&state.db)
    .await?;

    if !is_owner {
        return Err(AppError::Forbidden);
    }

    // Delete from EventMembers — clears both invite and RSVP
    sqlx::query(
        r#"DELETE FROM "EventMembers" WHERE "eventId" = $1 AND "userId" = $2"#
    )
    .bind(body.event_id)
    .bind(&body.user_id)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "success": true })))
}
