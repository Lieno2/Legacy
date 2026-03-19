use axum::{extract::{Query, State}, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};

use crate::{
    auth::AdminUser,
    error::{AppError, Result},
    models::{EventWithCreator, UserPublic},
    routes::AppState,
};

// ─── shared query params ───────────────────────────────────────────────────
#[derive(Deserialize, ToSchema)] pub struct IdQuery     { pub id: String }
#[derive(Deserialize, ToSchema)] pub struct EventIdQuery { pub id: i64    }

// ─── user requests ─────────────────────────────────────────────────────────
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

// ─── discord config ────────────────────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct DiscordConfig {
    pub webhook_url: String,
    pub enabled: bool,
    pub format: String,           // "embed" | "plain"
    pub msg_created: String,
    pub msg_updated: String,
    pub msg_deleted: String,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            webhook_url: String::new(),
            enabled: false,
            format: "embed".into(),
            msg_created: "📅 **{event.title}** has been created by {event.creator} on {event.date}{event.location}.".into(),
            msg_updated: "✏️ **{event.title}** was updated by {event.creator}. New date: {event.date}{event.location}.".into(),
            msg_deleted: "🗑️ **{event.title}** (was on {event.date}) has been deleted by {event.creator}.".into(),
        }
    }
}

// ─── stats ─────────────────────────────────────────────────────────────────
#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct MonthStat {
    pub month: String,
    pub count: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct ActiveUser {
    pub username: String,
    pub rsvp_count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RsvpBreakdown {
    pub going: i64,
    pub late: i64,
    pub not_going: i64,
    pub invited: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StatsResponse {
    pub events_per_month: Vec<MonthStat>,
    pub most_active_users: Vec<ActiveUser>,
    pub rsvp_breakdown: RsvpBreakdown,
}

// ─── audit log ─────────────────────────────────────────────────────────────
#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct AuditEntry {
    pub id: i64,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub entity_name: Option<String>,
    pub detail: Option<String>,
    pub snapshot: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct RevertRequest {
    pub audit_id: i64,
}

// ═══════════════════════════════════════════════════════════════════════════
// USER HANDLERS
// ═══════════════════════════════════════════════════════════════════════════

pub async fn list_users(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<UserPublic>>> {
    let users = sqlx::query_as::<_, UserPublic>(
        r#"SELECT id, username, email, perms,
           "createdAt" AT TIME ZONE 'UTC' AS created_at
           FROM "Users" ORDER BY "createdAt" ASC"#
    )
    .fetch_all(&state.db).await?;
    Ok(Json(users))
}

pub async fn create_user(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> Result<Json<UserPublic>> {
    let username = body.username.trim().to_string();
    let email    = body.email.trim().to_string();
    if username.is_empty() || email.is_empty() || body.password.is_empty() {
        return Err(AppError::BadRequest("Username, email and password are required".into()));
    }
    if body.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
    }
    let exists = sqlx::query(r#"SELECT id FROM "Users" WHERE email = $1"#)
        .bind(&email).fetch_optional(&state.db).await?.is_some();
    if exists { return Err(AppError::Conflict("Email already in use".into())); }
    let hash   = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST).map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let new_id = uuid::Uuid::new_v4().to_string();
    let perms  = body.perms.unwrap_or(0);
    let user   = sqlx::query_as::<_, UserPublic>(
        r#"INSERT INTO "Users" (id, username, email, "passwordHash", perms)
           VALUES ($1,$2,$3,$4,$5)
           RETURNING id, username, email, perms, "createdAt" AT TIME ZONE 'UTC' AS created_at"#
    )
    .bind(&new_id).bind(&username).bind(&email).bind(&hash).bind(perms)
    .fetch_one(&state.db).await?;
    Ok(Json(user))
}

pub async fn update_user(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<Json<UserPublic>> {
    let username = body.username.trim().to_string();
    let email    = body.email.trim().to_string();
    let perms    = body.perms.unwrap_or(0);
    if username.is_empty() || email.is_empty() {
        return Err(AppError::BadRequest("Username and email are required".into()));
    }
    let updated = if let Some(new_pw) = &body.new_password {
        if new_pw.len() < 8 { return Err(AppError::BadRequest("Password must be at least 8 characters".into())); }
        let hash = bcrypt::hash(new_pw, bcrypt::DEFAULT_COST).map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        sqlx::query_as::<_, UserPublic>(
            r#"UPDATE "Users" SET username=$1,email=$2,perms=$3,"passwordHash"=$4 WHERE id=$5
               RETURNING id,username,email,perms,"createdAt" AT TIME ZONE 'UTC' AS created_at"#
        ).bind(&username).bind(&email).bind(perms).bind(&hash).bind(&body.id)
        .fetch_optional(&state.db).await?.ok_or(AppError::NotFound)?
    } else {
        sqlx::query_as::<_, UserPublic>(
            r#"UPDATE "Users" SET username=$1,email=$2,perms=$3 WHERE id=$4
               RETURNING id,username,email,perms,"createdAt" AT TIME ZONE 'UTC' AS created_at"#
        ).bind(&username).bind(&email).bind(perms).bind(&body.id)
        .fetch_optional(&state.db).await?.ok_or(AppError::NotFound)?
    };
    Ok(Json(updated))
}

pub async fn delete_user(
    admin: AdminUser,
    State(state): State<AppState>,
    Query(q): Query<IdQuery>,
) -> Result<Json<serde_json::Value>> {
    if q.id == admin.0.sub { return Err(AppError::BadRequest("Cannot delete your own account".into())); }
    let deleted = sqlx::query(r#"DELETE FROM "Users" WHERE id=$1 RETURNING id"#)
        .bind(&q.id).fetch_optional(&state.db).await?.is_some();
    if !deleted { return Err(AppError::NotFound); }
    Ok(Json(serde_json::json!({ "success": true })))
}

// ═══════════════════════════════════════════════════════════════════════════
// EVENT HANDLERS
// ═══════════════════════════════════════════════════════════════════════════

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

pub async fn delete_event(
    admin: AdminUser,
    State(state): State<AppState>,
    Query(q): Query<EventIdQuery>,
) -> Result<Json<serde_json::Value>> {
    // Snapshot before delete for audit revert
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

    // Write audit entry
    sqlx::query(
        r#"INSERT INTO "AuditLog" (actor_id,actor_name,action,entity_type,entity_id,entity_name,detail,snapshot)
           VALUES ($1,$2,'delete','event',$3,$4,'Admin deleted event',$5)"#
    )
    .bind(&admin.0.sub)
    .bind(&admin.0.sub)  // actor_name fallback — ideally resolve username
    .bind(q.id.to_string())
    .bind(&entity_name)
    .bind(&snap_json)
    .execute(&state.db).await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

// ═══════════════════════════════════════════════════════════════════════════
// DISCORD CONFIG
// ═══════════════════════════════════════════════════════════════════════════

pub async fn get_discord(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<DiscordConfig>> {
    let row = sqlx::query_scalar::<_, String>(
        r#"SELECT value FROM "Settings" WHERE key='discord'"#
    ).fetch_optional(&state.db).await?;

    let cfg = match row {
        Some(json) => serde_json::from_str(&json).unwrap_or_default(),
        None       => DiscordConfig::default(),
    };
    Ok(Json(cfg))
}

pub async fn save_discord(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(body): Json<DiscordConfig>,
) -> Result<Json<serde_json::Value>> {
    let json = serde_json::to_string(&body).map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    sqlx::query(
        r#"INSERT INTO "Settings" (key,value) VALUES ('discord',$1)
           ON CONFLICT (key) DO UPDATE SET value=EXCLUDED.value"#
    ).bind(&json).execute(&state.db).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ═══════════════════════════════════════════════════════════════════════════
// STATS
// ═══════════════════════════════════════════════════════════════════════════

pub async fn get_stats(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<StatsResponse>> {
    let events_per_month = sqlx::query_as::<_, MonthStat>(
        r#"SELECT TO_CHAR(date,'YYYY-MM') AS month, COUNT(*)::BIGINT AS count
           FROM "Events"
           WHERE date >= NOW() - INTERVAL '12 months'
           GROUP BY month ORDER BY month ASC"#
    ).fetch_all(&state.db).await?;

    let most_active_users = sqlx::query_as::<_, ActiveUser>(
        r#"SELECT u.username, COUNT(em."userId")::BIGINT AS rsvp_count
           FROM "EventMembers" em
           JOIN "Users" u ON em."userId"=u.id
           WHERE em.status IN ('going','late')
           GROUP BY u.username ORDER BY rsvp_count DESC LIMIT 5"#
    ).fetch_all(&state.db).await?;

    let going     = sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM "EventMembers" WHERE status='going'"#).fetch_one(&state.db).await?;
    let late      = sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM "EventMembers" WHERE status='late'"#).fetch_one(&state.db).await?;
    let not_going = sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM "EventMembers" WHERE status='not_going'"#).fetch_one(&state.db).await?;
    let invited   = sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM "EventMembers" WHERE status='invited'"#).fetch_one(&state.db).await?;

    Ok(Json(StatsResponse {
        events_per_month,
        most_active_users,
        rsvp_breakdown: RsvpBreakdown { going, late, not_going, invited },
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// AUDIT LOG
// ═══════════════════════════════════════════════════════════════════════════

pub async fn list_audit(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<AuditEntry>>> {
    let entries = sqlx::query_as::<_, AuditEntry>(
        r#"SELECT id,actor_id,actor_name,action,entity_type,entity_id,entity_name,detail,snapshot,
           created_at AT TIME ZONE 'UTC' AS created_at
           FROM "AuditLog" ORDER BY created_at DESC LIMIT 200"#
    ).fetch_all(&state.db).await?;
    Ok(Json(entries))
}

pub async fn revert_audit(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(body): Json<RevertRequest>,
) -> Result<Json<serde_json::Value>> {
    let entry = sqlx::query_as::<_, AuditEntry>(
        r#"SELECT id,actor_id,actor_name,action,entity_type,entity_id,entity_name,detail,snapshot,
           created_at AT TIME ZONE 'UTC' AS created_at
           FROM "AuditLog" WHERE id=$1"#
    ).bind(body.audit_id).fetch_optional(&state.db).await?.ok_or(AppError::NotFound)?;

    if entry.action != "delete" || entry.entity_type != "event" {
        return Err(AppError::BadRequest("Only deleted events can be reverted".into()));
    }

    let snap = entry.snapshot.ok_or(AppError::BadRequest("No snapshot available".into()))?;
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

    // Mark audit entry as reverted
    sqlx::query(r#"UPDATE "AuditLog" SET detail=detail||' [REVERTED]' WHERE id=$1"#)
        .bind(body.audit_id).execute(&state.db).await?;

    Ok(Json(serde_json::json!({ "success": true })))
}
