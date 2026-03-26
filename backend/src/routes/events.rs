use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

use crate::{
    auth::AuthUser,
    error::{AppError, Result},
    models::EventWithCreator,
    routes::AppState,
};

use super::public_events::send_discord_notification;

#[derive(Deserialize, ToSchema)]
pub struct CreateEventRequest {
    pub title: String,
    pub description: Option<String>,
    pub date: DateTime<Utc>,
    pub location: Option<String>,
    pub color: Option<String>,
    pub private: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateEventRequest {
    pub title: String,
    pub description: Option<String>,
    pub date: DateTime<Utc>,
    pub location: Option<String>,
    pub color: Option<String>,
    pub private: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct PaginationQuery {
    /// Maximum number of events to return (default: 100, max: 500)
    pub limit: Option<i64>,
    /// Number of events to skip (default: 0)
    pub offset: Option<i64>,
}

/// List events visible to the current user
#[utoipa::path(
    get, path = "/api/events", tag = "Events",
    security(("bearer_auth" = [])),
    params(
        ("limit" = Option<i64>, Query, description = "Max results (default 100, max 500)"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination (default 0)"),
    ),
    responses((status = 200, body = Vec<EventWithCreator>), (status = 401))
)]
pub async fn list(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(page): Query<PaginationQuery>,
) -> Result<Json<Vec<EventWithCreator>>> {
    let limit  = page.limit.unwrap_or(100).min(500).max(1);
    let offset = page.offset.unwrap_or(0).max(0);

    let events = sqlx::query_as::<_, EventWithCreator>(
        r#"
        SELECT e.id, e.title, e.description,
               e.date AT TIME ZONE 'UTC' AS date,
               e.location, e.color,
               e."createdBy" AS created_by,
               e."createdAt" AT TIME ZONE 'UTC' AS created_at,
               e.private, u.username AS creator_name,
               e.share_token
        FROM "Events" e
        LEFT JOIN "Users" u ON e."createdBy" = u.id
        WHERE
            e.private = false
            OR e."createdBy" = $1
            OR EXISTS (
                SELECT 1 FROM "EventMembers" em
                WHERE em."eventId" = e.id AND em."userId" = $1
            )
        ORDER BY e.date ASC
        LIMIT $2 OFFSET $3
        "#
    )
    .bind(&auth.0.sub)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(events))
}

/// Create a new event
#[utoipa::path(
    post, path = "/api/events", tag = "Events",
    security(("bearer_auth" = [])), request_body = CreateEventRequest,
    responses((status = 200, body = EventWithCreator), (status = 400), (status = 401))
)]
pub async fn create(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateEventRequest>,
) -> Result<Json<EventWithCreator>> {
    if body.title.trim().is_empty() {
        return Err(AppError::BadRequest("Title is required".into()));
    }

    let is_private = body.private.unwrap_or(false);

    let event = sqlx::query_as::<_, EventWithCreator>(
        r#"
        WITH inserted AS (
            INSERT INTO "Events" (title, description, date, location, color, "createdBy", private, share_token)
            VALUES ($1, $2, $3, $4, $5, $6, $7, gen_random_uuid()::text)
            RETURNING *
        )
        SELECT i.id, i.title, i.description,
               i.date AT TIME ZONE 'UTC' AS date,
               i.location, i.color,
               i."createdBy" AS created_by,
               i."createdAt" AT TIME ZONE 'UTC' AS created_at,
               i.private, u.username AS creator_name,
               i.share_token
        FROM inserted i
        LEFT JOIN "Users" u ON i."createdBy" = u.id
        "#
    )
    .bind(body.title.trim())
    .bind(&body.description)
    .bind(body.date)
    .bind(&body.location)
    .bind(&body.color)
    .bind(&auth.0.sub)
    .bind(is_private)
    .fetch_one(&state.db)
    .await?;

    if !is_private {
        send_discord_notification(&state, "created", &event).await;
    }

    Ok(Json(event))
}

/// Update an existing event
#[utoipa::path(
    put, path = "/api/events/{id}", tag = "Events",
    security(("bearer_auth" = [])),
    params(("id" = i64, Path, description = "Event ID")),
    request_body = UpdateEventRequest,
    responses((status = 200, body = EventWithCreator), (status = 404))
)]
pub async fn update(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateEventRequest>,
) -> Result<Json<EventWithCreator>> {
    if body.title.trim().is_empty() {
        return Err(AppError::BadRequest("Title is required".into()));
    }

    let is_private = body.private.unwrap_or(false);

    let event = sqlx::query_as::<_, EventWithCreator>(
        r#"
        WITH updated AS (
            UPDATE "Events"
            SET title = $1, description = $2, date = $3, location = $4, color = $5, private = $6
            WHERE id = $7 AND "createdBy" = $8
            RETURNING *
        )
        SELECT u.id, u.title, u.description,
               u.date AT TIME ZONE 'UTC' AS date,
               u.location, u.color,
               u."createdBy" AS created_by,
               u."createdAt" AT TIME ZONE 'UTC' AS created_at,
               u.private, usr.username AS creator_name,
               u.share_token
        FROM updated u
        LEFT JOIN "Users" usr ON u."createdBy" = usr.id
        "#
    )
    .bind(body.title.trim())
    .bind(&body.description)
    .bind(body.date)
    .bind(&body.location)
    .bind(&body.color)
    .bind(is_private)
    .bind(id)
    .bind(&auth.0.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    if !is_private {
        send_discord_notification(&state, "updated", &event).await;
    }

    Ok(Json(event))
}

/// Delete an event
#[utoipa::path(
    delete, path = "/api/events/{id}", tag = "Events",
    security(("bearer_auth" = [])),
    params(("id" = i64, Path, description = "Event ID")),
    responses((status = 200, description = "Deleted"), (status = 404))
)]
pub async fn delete(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>> {
    let event = sqlx::query_as::<_, EventWithCreator>(
        r#"
        SELECT e.id, e.title, e.description,
               e.date AT TIME ZONE 'UTC' AS date,
               e.location, e.color,
               e."createdBy" AS created_by,
               e."createdAt" AT TIME ZONE 'UTC' AS created_at,
               e.private, u.username AS creator_name,
               e.share_token
        FROM "Events" e
        LEFT JOIN "Users" u ON e."createdBy" = u.id
        WHERE e.id = $1 AND e."createdBy" = $2
        "#
    )
    .bind(id)
    .bind(&auth.0.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    sqlx::query(r#"DELETE FROM "Events" WHERE id = $1 AND "createdBy" = $2"#)
        .bind(id)
        .bind(&auth.0.sub)
        .execute(&state.db)
        .await?;

    if !event.private {
        send_discord_notification(&state, "deleted", &event).await;
    }

    Ok(Json(serde_json::json!({ "success": true })))
}
