use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Users ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub perms: i16,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserPublic {
    pub id: String,
    pub username: String,
    pub email: String,
    pub perms: i16,
    pub created_at: Option<DateTime<Utc>>,
}

// ── Events ───────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Event {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub date: DateTime<Utc>,
    pub location: Option<String>,
    pub color: Option<String>,
    pub created_by: String,
    pub created_at: Option<DateTime<Utc>>,
    pub private: bool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventWithCreator {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub date: DateTime<Utc>,
    pub location: Option<String>,
    pub color: Option<String>,
    pub created_by: String,
    pub created_at: Option<DateTime<Utc>>,
    pub private: bool,
    pub creator_name: Option<String>,
}

// ── Event Members ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventMember {
    pub event_id: i64,
    pub user_id: String,
    pub username: Option<String>,
    pub status: String,
    pub late_minutes: Option<i32>,
    pub joined_at: Option<DateTime<Utc>>,
}

// ── JWT Claims ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,   // user id
    pub email: String,
    pub perms: i16,
    pub exp: usize,
    pub iat: usize,
}
