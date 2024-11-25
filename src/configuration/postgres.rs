use std::time::Duration;

use secrecy::{ExposeSecret, SecretBox};
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::postgres::PgSslMode;
use sqlx::ConnectOptions;
use sqlx::PgPool;
use tracing::log::LevelFilter;

#[derive(Deserialize)]
pub struct PostgresSettings {
    pub database: String,
    pub username: String,
    pub password: SecretBox<String>,
    pub port: u16,
    pub host: String,
    pub require_ssl: bool,
}

impl PostgresSettings {
    pub fn get_connect_options_without_database(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer // 首先尝试SSL连接; 如果失败, 则尝试非SSL连接
        };

        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .ssl_mode(ssl_mode)
    }

    pub fn get_connect_options_with_database(&self) -> PgConnectOptions {
        self.get_connect_options_without_database()
            .database(&self.database)
            .log_statements(LevelFilter::Trace)
    }

    pub fn get_postgres_connection_pool(&self) -> PgPool {
        PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(3))
            .connect_lazy_with(self.get_connect_options_with_database()) // 只在首次使用连接池时才会尝试建立连接
    }
}
