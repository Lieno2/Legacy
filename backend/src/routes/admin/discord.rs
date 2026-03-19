use axum::{extract::State, Json};

use crate::{
    auth::AdminUser,
    error::{AppError, Result},
    routes::AppState,
};
use super::DiscordConfig;

#[utoipa::path(get, path = "/api/admin/discord", tag = "Admin",
    security(("bearer_auth" = [])),
    responses((status = 200, body = DiscordConfig)))]
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

#[utoipa::path(post, path = "/api/admin/discord", tag = "Admin",
    security(("bearer_auth" = [])), request_body = DiscordConfig,
    responses((status = 200, description = "Saved")))]
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
