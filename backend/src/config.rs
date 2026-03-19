#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub access_token_expiry_secs: u64,
    pub refresh_token_expiry_secs: u64,
    pub frontend_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            redis_url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            access_token_expiry_secs: std::env::var("ACCESS_TOKEN_EXPIRY_SECS")
                .unwrap_or_else(|_| "900".to_string())
                .parse()
                .unwrap_or(900),
            refresh_token_expiry_secs: std::env::var("REFRESH_TOKEN_EXPIRY_SECS")
                .unwrap_or_else(|_| "604800".to_string())
                .parse()
                .unwrap_or(604800),
            frontend_url: std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string()),
        }
    }
}
