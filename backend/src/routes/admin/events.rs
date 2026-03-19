use axum::{extract::{Query, State}, Json};

use crate::{
    auth::AdminUser,
    error::{AppError, Result},
    models::EventWithCreator,
    routes::AppState,
};
use super::EventIdQuery;

#[utoipa::path(get, path = "/api/admin/events", tag = "Admin",
    security(("bearer_auth" = [])),
    responses((status = 200, body = Vec<EventWithCreator>), (status = 403, description = "Forbidden")))]
pub async fn list_events(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<EventWithCreator>>> {
    let events = sqlx::query_as::<_, EventWithCreator>(
        r#"SELECT e.id,e.title,e.description,e.date AT TIME ZONE 'UTC' AS date,
           e.location,e.color,e."createdBy" AS created_by,
           e."createdAt" AT TIME ZONE 'UTC' AS created_at,
           e.private,u.username AS creator_name
           FROM "Events" e LEFT JOIN "Users" u ON e."createdBy"=u.id
           ORDER BY e.date ASC"#
    ).fetch_all(&state.db).await?;
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
           e.private,u.username AS creator_name
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
