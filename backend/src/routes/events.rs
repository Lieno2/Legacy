use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use chrono::{DateTime, Utc};

use crate::{
    auth::AuthUser,
    error::{AppError, Result},
    models::EventWithCreator,
    routes::AppState,
};

#[derive(Deserialize)]
pub struct CreateEventRequest {
    pub title: String,
    pub description: Option<String>,
    pub date: DateTime<Utc>,
    pub location: Option<String>,
    pub color: Option<String>,
    pub private: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateEventRequest {
    pub title: String,
    pub description: Option<String>,
    pub date: DateTime<Utc>,
    pub location: Option<String>,
    pub color: Option<String>,
    pub private: Option<bool>,
}

pub async fn list(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<EventWithCreator>>> {
    let events = sqlx::query_as::<_, EventWithCreator>(
        r#"
        SELECT e.id, e.title, e.description, e.date, e.location, e.color,
               e."createdBy" AS created_by, e."createdAt" AS created_at,
               e.private, u.username AS creator_name
        FROM "Events" e
        LEFT JOIN "Users" u ON e."createdBy" = u.id
        WHERE e."createdBy" = $1

        UNION

        SELECT e.id, e.title, e.description, e.date, e.location, e.color,
               e."createdBy" AS created_by, e."createdAt" AS created_at,
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
        SELECT i.id, i.title, i.description, i.date, i.location, i.color,
               i."createdBy" AS created_by, i."createdAt" AS created_at,
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
        SELECT u.id, u.title, u.description, u.date, u.location, u.color,
               u."createdBy" AS created_by, u."createdAt" AS created_at,
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
