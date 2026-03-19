use sqlx::PgPool;
use crate::config::Config;

pub async fn run_setup(db: &PgPool, cfg: &Config) {
    if cfg.setup_account_enabled {
        let hash = match bcrypt::hash(&cfg.setup_account_password, bcrypt::DEFAULT_COST) {
            Ok(h) => h,
            Err(e) => {
                tracing::error!("Setup account: failed to hash password: {}", e);
                return;
            }
        };

        let result = sqlx::query(
            r#"
            INSERT INTO "Users" (id, username, email, "passwordHash", perms)
            VALUES ('setup-account', 'Setup Admin', $1, $2, 999)
            ON CONFLICT (id) DO UPDATE
                SET email = EXCLUDED.email,
                    "passwordHash" = EXCLUDED."passwordHash"
            "#
        )
        .bind(&cfg.setup_account_email)
        .bind(&hash)
        .execute(db)
        .await;

        match result {
            Ok(_) => tracing::info!("Setup account enabled: {}", cfg.setup_account_email),
            Err(e) => tracing::error!("Setup account: failed to upsert user: {}", e),
        }
    } else {
        // Remove setup account if it exists
        let result = sqlx::query(r#"DELETE FROM "Users" WHERE id = 'setup-account'"#)
            .execute(db)
            .await;

        match result {
            Ok(r) if r.rows_affected() > 0 => tracing::info!("Setup account disabled and removed"),
            Ok(_) => {}
            Err(e) => tracing::error!("Setup account: failed to remove user: {}", e),
        }
    }
}
