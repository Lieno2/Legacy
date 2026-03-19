pub mod users;
pub mod events;
pub mod discord;
pub mod stats;
pub mod audit;

// Re-export handlers so routes/mod.rs call-sites stay unchanged
pub use users::{list_users, create_user, update_user, delete_user};
pub use events::{list_events, delete_event};
pub use discord::{get_discord, save_discord};
pub use stats::get_stats;
pub use audit::{list_audit, revert_audit};

// ── Shared types ────────────────────────────────────────────────────────────
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)] pub struct IdQuery     { pub id: String }
#[derive(Deserialize, ToSchema)] pub struct EventIdQuery { pub id: i64    }

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

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct DiscordConfig {
    pub webhook_url: String,
    pub enabled: bool,
    pub format: String,
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
            msg_created: "\u{1F4C5} **{event.title}** has been created by {event.creator} on {event.date}{event.location}.".into(),
            msg_updated: "\u{270F}\u{FE0F} **{event.title}** was updated by {event.creator}. New date: {event.date}{event.location}.".into(),
            msg_deleted: "\u{1F5D1}\u{FE0F} **{event.title}** (was on {event.date}) has been deleted by {event.creator}.".into(),
        }
    }
}

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

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct AuditEntry {
    pub id: i64,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub action: String,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub target_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct RevertRequest {
    pub audit_id: i64,
}
