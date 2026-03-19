use axum::{extract::{Query, State}, Json};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    auth::AuthUser,
    error::{AppError, Result},
    models::EventMember,
    routes::AppState,
};

#[derive(Deserialize, ToSchema)]
pub struct EventIdQuery {
    pub event_id: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct RsvpRequest {
    pub event_id: i64,
    pub status: String,
    pub late_minutes: Option<i32>,
}

/// List RSVPs for an event
#[utoipa::path(
    get,
    path = "/api/rsvp",
    tag = "RSVP",
    security(("bearer_auth" = [])),
    params(("event_id" = i64, Query, description = "Event ID")),
    responses(
        (status = 200, description = "List of RSVPs", body = Vec<EventMember>),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list(
    _auth: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<EventIdQuery>,
) -> Result<Json<Vec<EventMember>>> {
    let members = sqlx::query_as::<_, EventMember>(
        r#"
        SELECT em."eventId" AS event_id, em."userId" AS user_id,
               u.username, em.status, em."lateMinutes" AS late_minutes, em."joinedAt" AS joined_at
        FROM "EventMembers" em
        LEFT JOIN "Users" u ON em."userId" = u.id
        WHERE em."eventId" = $1
        "#
    )
    .bind(q.event_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(members))
}

/// Upsert RSVP for an event
#[utoipa::path(
    post,
    path = "/api/rsvp",
    tag = "RSVP",
    security(("bearer_auth" = [])),
    request_body = RsvpRequest,
    responses(
        (status = 200, description = "RSVP saved"),
        (status = 400, description = "Invalid status"),
        (status = 404, description = "Event not found"),
    )
)]
pub async fn upsert(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<RsvpRequest>,
) -> Result<Json<serde_json::Value>> {
    if !["going", "late", "not_going"].contains(&body.status.as_str()) {
        return Err(AppError::BadRequest("Invalid status".into()));
    }

    let exists = sqlx::query(r#"SELECT id FROM "Events" WHERE id = $1"#)
        .bind(body.event_id)
        .fetch_optional(&state.db)
        .await?
        .is_some();
    if !exists {
        return Err(AppError::NotFound);
    }

    let late_minutes = if body.status == "late" { body.late_minutes } else { None };

    sqlx::query(
        r#"
        INSERT INTO "EventMembers" ("eventId", "userId", status, "lateMinutes")
        VALUES ($1, $2, $3, $4)
        ON CONFLICT ("eventId", "userId") DO UPDATE
            SET status = EXCLUDED.status, "lateMinutes" = EXCLUDED."lateMinutes"
        "#
    )
    .bind(body.event_id)
    .bind(&auth.0.sub)
    .bind(&body.status)
    .bind(late_minutes)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Remove RSVP for an event
#[utoipa::path(
    delete,
    path = "/api/rsvp",
    tag = "RSVP",
    security(("bearer_auth" = [])),
    params(("event_id" = i64, Query, description = "Event ID")),
    responses(
        (status = 200, description = "RSVP removed"),
    )
)]
pub async fn remove(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<EventIdQuery>,
) -> Result<Json<serde_json::Value>> {
    sqlx::query(
        r#"DELETE FROM "EventMembers" WHERE "eventId" = $1 AND "userId" = $2"#
    )
    .bind(q.event_id)
    .bind(&auth.0.sub)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "success": true })))
}
