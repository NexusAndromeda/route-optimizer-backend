//! Configuración de base de datos
//! 
//! Este módulo maneja la conexión y configuración de PostgreSQL con SQLx.

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Configuración de la base de datos
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set in environment variables"),
            max_connections: 20,
            min_connections: 5,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(3600),
        }
    }
}

impl DatabaseConfig {
    /// Crear un nuevo pool de conexiones
    pub async fn create_pool(&self) -> Result<PgPool, sqlx::Error> {
        PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .idle_timeout(self.idle_timeout)
            .max_lifetime(self.max_lifetime)
            .connect(&self.url)
            .await
    }

    /// Crear un pool de conexiones para testing
    pub async fn create_test_pool(&self) -> Result<PgPool, sqlx::Error> {
        PgPoolOptions::new()
            .max_connections(5)
            .min_connections(1)
            .idle_timeout(Duration::from_secs(60))
            .max_lifetime(Duration::from_secs(300))
            .connect(&self.url)
            .await
    }
}

