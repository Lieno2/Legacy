use axum::{extract::State, Json};

use crate::{
    auth::AdminUser,
    error::{AppError, Result},
    models::EventWithCreator,
    routes::AppState,
};
use super::{AuditEntry, RevertRequest};

#[utoipa::path(get, path = "/api/admin/audit", tag = "Admin",
    security(("bearer_auth" = [])),
    responses((status = 200, body = Vec<AuditEntry>)))]
pub async fn list_audit(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<AuditEntry>>> {
    let entries = sqlx::query_as::<_, AuditEntry>(
        r#"SELECT id,
           "userId"     AS user_id,
           username,
           action,
           "targetType" AS target_type,
           "targetId"   AS target_id,
           "targetName" AS target_name,
           metadata,
           "createdAt" AT TIME ZONE 'UTC' AS created_at
           FROM "AuditLog" ORDER BY "createdAt" DESC LIMIT 200"#
    ).fetch_all(&state.db).await?;
    Ok(Json(entries))
}

#[utoipa::path(post, path = "/api/admin/audit/revert", tag = "Admin",
    security(("bearer_auth" = [])), request_body = RevertRequest,
    responses((status = 200, description = "Reverted"), (status = 400, description = "Bad request")))]
pub async fn revert_audit(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(body): Json<RevertRequest>,
) -> Result<Json<serde_json::Value>> {
    let entry = sqlx::query_as::<_, AuditEntry>(
        r#"SELECT id,
           "userId"     AS user_id,
           username,
           action,
           "targetType" AS target_type,
           "targetId"   AS target_id,
           "targetName" AS target_name,
           metadata,
           "createdAt" AT TIME ZONE 'UTC' AS created_at
           FROM "AuditLog" WHERE id=$1"#
    ).bind(body.audit_id).fetch_optional(&state.db).await?.ok_or(AppError::NotFound)?;

    if entry.action != "delete" || entry.target_type.as_deref() != Some("event") {
        return Err(AppError::BadRequest("Only deleted events can be reverted".into()));
    }

    let snap = entry.metadata.ok_or(AppError::BadRequest("No snapshot available".into()))?;
    let ev: EventWithCreator = serde_json::from_value(snap)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    sqlx::query(
        r#"INSERT INTO "Events" (id,title,description,date,location,color,"createdBy",private)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
           ON CONFLICT (id) DO NOTHING"#
    )
    .bind(ev.id)
    .bind(&ev.title)
    .bind(&ev.description)
    .bind(ev.date)
    .bind(&ev.location)
    .bind(&ev.color)
    .bind(&ev.created_by)
    .bind(ev.private)
    .execute(&state.db).await?;

    sqlx::query(
        r#"UPDATE "AuditLog" SET metadata = COALESCE(metadata,'{}') || '{"reverted":true}' WHERE id=$1"#
    ).bind(body.audit_id).execute(&state.db).await?;

    Ok(Json(serde_json::json!({ "success": true })))
}
