use axum::extract::{Path, State};
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

/// List events for the current user
#[utoipa::path(
    get,
    path = "/api/events",
    tag = "Events",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of events", body = Vec<EventWithCreator>),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<EventWithCreator>>> {
    let events = sqlx::query_as::<_, EventWithCreator>(
        r#"
        SELECT e.id, e.title, e.description,
               e.date AT TIME ZONE 'UTC' AS date,
               e.location, e.color,
               e."createdBy" AS created_by,
               e."createdAt" AT TIME ZONE 'UTC' AS created_at,
               e.private, u.username AS creator_name
        FROM "Events" e
        LEFT JOIN "Users" u ON e."createdBy" = u.id
        WHERE e."createdBy" = $1

        UNION

        SELECT e.id, e.title, e.description,
               e.date AT TIME ZONE 'UTC' AS date,
               e.location, e.color,
               e."createdBy" AS created_by,
               e."createdAt" AT TIME ZONE 'UTC' AS created_at,
               e.private, u.username AS creator_name
        FROM "EventMembers" em
        INNER JOIN "Events" e ON em."eventId" = e.id
        LEFT JOIN "Users" u ON e."createdBy" = u.id
        WHERE em."userId" = $1 AND e.private = false

        ORDER BY date ASC
        "#
    )
    .bind(&auth.0.sub)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(events))
}

/// Create a new event
#[utoipa::path(
    post,
    path = "/api/events",
    tag = "Events",
    security(("bearer_auth" = [])),
    request_body = CreateEventRequest,
    responses(
        (status = 200, description = "Created event", body = EventWithCreator),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn create(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateEventRequest>,
) -> Result<Json<EventWithCreator>> {
    if body.title.trim().is_empty() {
        return Err(AppError::BadRequest("Title is required".into()));
    }

    let event = sqlx::query_as::<_, EventWithCreator>(
        r#"
        WITH inserted AS (
            INSERT INTO "Events" (title, description, date, location, color, "createdBy", private)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
        )
        SELECT i.id, i.title, i.description,
               i.date AT TIME ZONE 'UTC' AS date,
               i.location, i.color,
               i."createdBy" AS created_by,
               i."createdAt" AT TIME ZONE 'UTC' AS created_at,
               i.private, u.username AS creator_name
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
    .bind(body.private.unwrap_or(false))
    .fetch_one(&state.db)
    .await?;

    Ok(Json(event))
}

/// Update an existing event
#[utoipa::path(
    put,
    path = "/api/events/{id}",
    tag = "Events",
    security(("bearer_auth" = [])),
    params(("id" = i64, Path, description = "Event ID")),
    request_body = UpdateEventRequest,
    responses(
        (status = 200, description = "Updated event", body = EventWithCreator),
        (status = 404, description = "Event not found"),
    )
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
               u.private, usr.username AS creator_name
        FROM updated u
        LEFT JOIN "Users" usr ON u."createdBy" = usr.id
        "#
    )
    .bind(body.title.trim())
    .bind(&body.description)
    .bind(body.date)
    .bind(&body.location)
    .bind(&body.color)
    .bind(body.private.unwrap_or(false))
    .bind(id)
    .bind(&auth.0.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(event))
}

/// Delete an event
#[utoipa::path(
    delete,
    path = "/api/events/{id}",
    tag = "Events",
    security(("bearer_auth" = [])),
    params(("id" = i64, Path, description = "Event ID")),
    responses(
        (status = 200, description = "Deleted"),
        (status = 404, description = "Event not found"),
    )
)]
pub async fn delete(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>> {
    let deleted = sqlx::query(
        r#"DELETE FROM "Events" WHERE id = $1 AND "createdBy" = $2 RETURNING id"#
    )
    .bind(id)
    .bind(&auth.0.sub)
    .fetch_optional(&state.db)
    .await?;

    if deleted.is_none() {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "success": true })))
}
