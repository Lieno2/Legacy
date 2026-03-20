use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

use crate::{
    error::{AppError, Result},
    models::EventWithCreator,
    routes::AppState,
};

// ── Public share types ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct PublicEvent {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub date: DateTime<Utc>,
    pub location: Option<String>,
    pub color: Option<String>,
    pub creator_name: Option<String>,
    pub rsvp_counts: RsvpCounts,
}

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct RsvpCounts {
    pub going: i64,
    pub late: i64,
    pub not_going: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct PublicEventRow {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub date: DateTime<Utc>,
    pub location: Option<String>,
    pub color: Option<String>,
    pub private: bool,
    pub creator_name: Option<String>,
}

/// Get a public event by its share token — no auth required
#[utoipa::path(
    get,
    path = "/api/events/public/{token}",
    tag = "Events",
    params(("token" = String, Path, description = "Share token")),
    responses(
        (status = 200, description = "Event details"),
        (status = 404, description = "Not found or private"),
    )
)]
pub async fn get_public(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<PublicEvent>> {
    let row = sqlx::query_as::<_, PublicEventRow>(
        r#"
        SELECT e.id, e.title, e.description,
               e.date AT TIME ZONE 'UTC' AS date,
               e.location, e.color, e.private,
               u.username AS creator_name
        FROM "Events" e
        LEFT JOIN "Users" u ON e."createdBy" = u.id
        WHERE e.share_token = $1
        "#
    )
    .bind(&token)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    if row.private {
        return Err(AppError::NotFound);
    }

    let counts = sqlx::query_as::<_, RsvpCounts>(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE status = 'going')     AS going,
            COUNT(*) FILTER (WHERE status = 'late')      AS late,
            COUNT(*) FILTER (WHERE status = 'not_going') AS not_going
        FROM "EventMembers"
        WHERE "eventId" = $1
        "#
    )
    .bind(row.id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(PublicEvent {
        id: row.id,
        title: row.title,
        description: row.description,
        date: row.date,
        location: row.location,
        color: row.color,
        creator_name: row.creator_name,
        rsvp_counts: counts,
    }))
}

// ── Discord notification helper ───────────────────────────────────────────────

pub async fn send_discord_notification(
    state: &AppState,
    action: &str,
    event: &EventWithCreator,
) {
    let cfg = match get_discord_config(state).await {
        Some(c) if c.enabled && !c.webhook_url.is_empty() => c,
        _ => return,
    };

    if !cfg.webhook_url.starts_with("https://discord.com/api/webhooks/") {
        return;
    }

    let date_str = event.date.format("%A, %d %B %Y").to_string();
    let location_str = event.location.as_deref()
        .map(|l| format!("\n📍 {}", l))
        .unwrap_or_default();

    let template = match action {
        "created" => &cfg.msg_created,
        "updated" => &cfg.msg_updated,
        "deleted" => &cfg.msg_deleted,
        _ => return,
    };

    let content = template
        .replace("{event.title}",    &event.title)
        .replace("{event.creator}",  event.creator_name.as_deref().unwrap_or("Unknown"))
        .replace("{event.date}",     &date_str)
        .replace("{event.location}", &location_str);

    let body = if cfg.format == "embed" {
        let color_int = hex_to_discord_color(event.color.as_deref());
        let (title_prefix, embed_color) = match action {
            "created" => ("📅 ", color_int.unwrap_or(0x10b981u32)),
            "updated" => ("✏️ ", color_int.unwrap_or(0xf59e0bu32)),
            "deleted" => ("🗑️ ", color_int.unwrap_or(0xf43f5eu32)),
            _         => ("",    0x6366f1u32),
        };

        let mut fields = vec![
            serde_json::json!({ "name": "📆 Date", "value": &date_str, "inline": true })
        ];
        if let Some(loc) = &event.location {
            fields.push(serde_json::json!({ "name": "📍 Location", "value": loc, "inline": true }));
        }

        serde_json::json!({
            "embeds": [{
                "title": format!("{}{}", title_prefix, event.title),
                "description": content,
                "color": embed_color,
                "fields": fields,
                "footer": {
                    "text": format!("Legacy Calendar • by {}", event.creator_name.as_deref().unwrap_or("Unknown"))
                },
                "timestamp": event.date.to_rfc3339()
            }]
        })
    } else {
        serde_json::json!({ "content": content })
    };

    let client = reqwest::Client::new();
    if let Err(e) = client.post(&cfg.webhook_url).json(&body).send().await {
        tracing::warn!("Discord webhook failed: {}", e);
    }
}

async fn get_discord_config(state: &AppState) -> Option<crate::routes::admin::DiscordConfig> {
    let row = sqlx::query_scalar::<_, String>(
        r#"SELECT value FROM "Settings" WHERE key='discord'"#,
    )
    .fetch_optional(&state.db)
    .await
    .ok()??;

    serde_json::from_str(&row).ok()
}

fn hex_to_discord_color(hex: Option<&str>) -> Option<u32> {
    let hex = hex?.trim_start_matches('#');
    u32::from_str_radix(hex, 16).ok()
}