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

        // If a user with this email already exists, just update their password
        // and ensure they have admin perms — never touch their ID (would break FKs)
        let result = sqlx::query(
            r#"
            INSERT INTO "Users" (id, username, email, "passwordHash", perms)
            VALUES (gen_random_uuid()::text, 'Admin', $1, $2, 999)
            ON CONFLICT (email) DO UPDATE
                SET "passwordHash" = EXCLUDED."passwordHash",
                    perms = 999
            "#
        )
        .bind(&cfg.setup_account_email)
        .bind(&hash)
        .execute(db)
        .await;

        match result {
            Ok(_)  => tracing::info!("✅ Setup account ready: {}", cfg.setup_account_email),
            Err(e) => tracing::error!("Setup account: failed to upsert user: {}", e),
        }
    } else {
        // Only delete if no events reference this account
        // (delete only the setup-account id row that has no events)
        let result = sqlx::query(
            r#"
            DELETE FROM "Users"
            WHERE id = 'setup-account'
              AND NOT EXISTS (
                SELECT 1 FROM "Events" WHERE "createdBy" = 'setup-account'
              )
            "#
        )
        .execute(db)
        .await;

        match result {
            Ok(r) if r.rows_affected() > 0 => tracing::info!("Setup account disabled and removed"),
            Ok(_)  => {}
            Err(e) => tracing::error!("Setup account: failed to remove user: {}", e),
        }
    }
}