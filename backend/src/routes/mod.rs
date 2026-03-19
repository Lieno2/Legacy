pub mod auth;
pub mod events;
pub mod rsvp;
pub mod account;
pub mod admin;

use axum::{Router, routing::{get, post, put, delete}};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: redis::Client,
    pub cfg: crate::config::Config,
}

pub fn all_routes() -> Router<AppState> {
    Router::new()
        // Auth
        .route("/auth/login",   post(auth::login))
        .route("/auth/logout",  post(auth::logout))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/me",      get(auth::me))
        // Events
        .route("/events",       get(events::list).post(events::create))
        .route("/events/:id",   put(events::update).delete(events::delete))
        // RSVP
        .route("/rsvp",         get(rsvp::list).post(rsvp::upsert).delete(rsvp::remove))
        // Account
        .route("/account",      get(account::get_profile).put(account::update_profile))
        // Admin
        .route("/admin/users",  get(admin::list_users).post(admin::create_user).put(admin::update_user).delete(admin::delete_user))
        .route("/admin/events", get(admin::list_events).delete(admin::delete_event))
}
