mod config;
mod db;
mod error;
mod models;
mod routes;
mod auth;
mod setup;

use axum::{
    Router,
    http::{HeaderValue, Method, header},
};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use routes::{auth as auth_routes, events, rsvp, account, admin, invites, polls};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Legacy API",
        version = "1.0.0",
        description = "Backend API for Legacy calendar app"
    ),
    paths(
        auth_routes::login,
        auth_routes::logout,
        auth_routes::refresh,
        auth_routes::me,
        events::list,
        events::create,
        events::update,
        events::delete,
        rsvp::list,
        rsvp::upsert,
        rsvp::remove,
        account::get_profile,
        account::update_profile,
        admin::users::list_users,
        admin::users::create_user,
        admin::users::update_user,
        admin::users::delete_user,
        admin::events::list_events,
        admin::events::delete_event,
        admin::discord::get_discord,
        admin::discord::save_discord,
        admin::stats::get_stats,
        admin::audit::list_audit,
        admin::audit::revert_audit,
        invites::search_users,
        invites::list,
        invites::invite,
        invites::remove,
        polls::get_poll,
        polls::upsert_poll,
        polls::delete_poll,
        polls::answer_poll,
        polls::get_voters,
    ),
    components(
        schemas(
            models::UserPublic,
            models::EventWithCreator,
            models::EventMember,
            auth_routes::LoginRequest,
            auth_routes::AuthResponse,
            auth_routes::RefreshRequest,
            auth_routes::LogoutRequest,
            events::CreateEventRequest,
            events::UpdateEventRequest,
            rsvp::RsvpRequest,
            rsvp::EventIdQuery,
            account::UpdateProfileRequest,
            admin::CreateUserRequest,
            admin::UpdateUserRequest,
            admin::IdQuery,
            admin::EventIdQuery,
            admin::DiscordConfig,
            admin::StatsResponse,
            admin::MonthStat,
            admin::ActiveUser,
            admin::RsvpBreakdown,
            admin::AuditEntry,
            admin::RevertRequest,
            invites::InviteUser,
            invites::InviteRequest,
            invites::RemoveInviteRequest,
            polls::PollChoice,
            polls::PollResponse,
            polls::CreatePollRequest,
            polls::AnswerPollRequest,
            polls::VoterEntry,
            polls::ChoiceVoters,
        )
    ),
    tags(
        (name = "Auth",    description = "Authentication endpoints"),
        (name = "Events",  description = "Event management"),
        (name = "RSVP",    description = "RSVP management"),
        (name = "Account", description = "User account"),
        (name = "Admin",   description = "Admin only endpoints"),
        (name = "Invites", description = "Private event invite management"),
        (name = "Polls",   description = "Event polls"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    )
                ),
            );
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "legacy_backend=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let cfg = config::Config::from_env();

    let pg_pool = db::create_pg_pool(&cfg.database_url).await;
    let redis_client = db::create_redis_client(&cfg.redis_url);

    setup::run_setup(&pg_pool, &cfg).await;

    let cors = CorsLayer::new()
        .allow_origin(cfg.frontend_url.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
        .allow_credentials(true);

    let app = Router::new()
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
        .nest("/api", routes::all_routes())
        .layer(cors)
        .with_state(routes::AppState {
            db: pg_pool,
            redis: redis_client,
            cfg,
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    tracing::info!("Backend listening on http://0.0.0.0:3001");
    tracing::info!("Scalar docs at http://0.0.0.0:3001/scalar");
    axum::serve(listener, app).await.unwrap();
}
