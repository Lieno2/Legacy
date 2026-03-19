mod config;
mod db;
mod error;
mod models;
mod routes;
mod auth;

use axum::{
    Router,
    http::{HeaderValue, Method, header},
};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    let cors = CorsLayer::new()
        .allow_origin(cfg.frontend_url.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
        .allow_credentials(true);

    let app = Router::new()
        .nest("/api", routes::all_routes())
        .layer(cors)
        .with_state(routes::AppState {
            db: pg_pool,
            redis: redis_client,
            cfg,
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    tracing::info!("Backend listening on http://0.0.0.0:3001");
    axum::serve(listener, app).await.unwrap();
}
