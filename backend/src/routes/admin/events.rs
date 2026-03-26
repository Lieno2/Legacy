use axum::{extract::{Query, State}, Json};
use serde::Deserialize;

use crate::{
    auth::AdminUser,
    error::{AppError, Result},
    models::EventWithCreator,
    routes::AppState,
};
use super::EventIdQuery;

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[utoipa::path(get, path = "/api/admin/events", tag = "Admin",
    security(("bearer_auth" = [])),
    params(
        ("limit" = Option<i64>, Query, description = "Max results (default 100, max 500)"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination (default 0)"),
    ),
    responses((status = 200, body = Vec<EventWithCreator>), (status = 403, description = "Forbidden")))]
pub async fn list_events(
    _admin: AdminUser,
    State(state): State<AppState>,
    Query(page): Query<PaginationQuery>,
) -> Result<Json<Vec<EventWithCreator>>> {
    let limit  = page.limit.unwrap_or(100).min(500).max(1);
    let offset = page.offset.unwrap_or(0).max(0);

    let events = sqlx::query_as::<_, EventWithCreator>(
        r#"SELECT e.id,e.title,e.description,e.date AT TIME ZONE 'UTC' AS date,
           e.location,e.color,e."createdBy" AS created_by,
           e."createdAt" AT TIME ZONE 'UTC' AS created_at,
           e.private,u.username AS creator_name,
           e.share_token
           FROM "Events" e LEFT JOIN "Users" u ON e."createdBy"=u.id
           ORDER BY e.date ASC
           LIMIT $1 OFFSET $2"#
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db).await?;
    Ok(Json(events))
}

#[utoipa::path(delete, path = "/api/admin/events", tag = "Admin",
    security(("bearer_auth" = [])),
    params(("id" = i64, Query, description = "Event ID")),
    responses((status = 200, description = "Deleted"), (status = 404, description = "Not found")))]
pub async fn delete_event(
    admin: AdminUser,
    State(state): State<AppState>,
    Query(q): Query<EventIdQuery>,
) -> Result<Json<serde_json::Value>> {
    let snapshot = sqlx::query_as::<_, EventWithCreator>(
        r#"SELECT e.id,e.title,e.description,e.date AT TIME ZONE 'UTC' AS date,
           e.location,e.color,e."createdBy" AS created_by,
           e."createdAt" AT TIME ZONE 'UTC' AS created_at,
           e.private,u.username AS creator_name,
           e.share_token
           FROM "Events" e LEFT JOIN "Users" u ON e."createdBy"=u.id
           WHERE e.id=$1"#
    ).bind(q.id).fetch_optional(&state.db).await?;

    let (entity_name, snap_json) = match &snapshot {
        Some(ev) => (Some(ev.title.clone()), serde_json::to_value(ev).ok()),
        None     => return Err(AppError::NotFound),
    };

    sqlx::query(r#"DELETE FROM "Events" WHERE id=$1"#)
        .bind(q.id).execute(&state.db).await?;

    let actor_username = sqlx::query_scalar::<_, String>(
        r#"SELECT username FROM "Users" WHERE id=$1"#
    ).bind(&admin.0.sub).fetch_optional(&state.db).await?
     .unwrap_or_else(|| admin.0.sub.clone());

    sqlx::query(
        r#"INSERT INTO "AuditLog" ("userId",username,action,"targetType","targetId","targetName",metadata)
           VALUES ($1,$2,'delete','event',$3,$4,$5)"#
    )
    .bind(&admin.0.sub)
    .bind(&actor_username)
    .bind(q.id.to_string())
    .bind(&entity_name)
    .bind(&snap_json)
    .execute(&state.db).await?;

    Ok(Json(serde_json::json!({ "success": true })))
}
