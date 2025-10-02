//! Módulo de conexión a la base de datos
//! 
//! Maneja la conexión y configuración de PostgreSQL

use anyhow::Result;
use sqlx::PgPool;
use tracing::{info, error};

/// Configuración de la base de datos
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://core:password@localhost:5432/route_optimizer".to_string()),
            max_connections: 10,
            min_connections: 1,
            acquire_timeout: 30,
            idle_timeout: 600,
        }
    }
}

/// Estructura para manejar la conexión a la base de datos
pub struct DatabaseConnection {
    pub pool: PgPool,
}

impl DatabaseConnection {
    /// Crear nueva conexión a la base de datos
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        info!("🔗 Conectando a la base de datos...");
        
        let pool = sqlx::PgPool::connect(&config.url).await?;
        
        // Test de conexión
        sqlx::query("SELECT 1").fetch_one(&pool).await?;
        
        info!("✅ Base de datos conectada exitosamente");
        
        Ok(Self { pool })
    }
    
    /// Crear conexión con configuración por defecto
    pub async fn new_default() -> Result<Self> {
        Self::new(DatabaseConfig::default()).await
    }
    
    /// Obtener el pool de conexiones
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
    
    /// Verificar que la conexión esté activa
    pub async fn health_check(&self) -> Result<bool> {
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => Ok(true),
            Err(e) => {
                error!("❌ Error en health check de la base de datos: {}", e);
                Ok(false)
            }
        }
    }
    
    /// Obtener información de la base de datos
    pub async fn get_database_info(&self) -> Result<DatabaseInfo> {
        let version: String = sqlx::query_scalar("SELECT version()").fetch_one(&self.pool).await?;
        let current_database: String = sqlx::query_scalar("SELECT current_database()").fetch_one(&self.pool).await?;
        let current_user: String = sqlx::query_scalar("SELECT current_user").fetch_one(&self.pool).await?;
        
        Ok(DatabaseInfo {
            version,
            database_name: current_database,
            current_user,
        })
    }
}

/// Información de la base de datos
#[derive(Debug, Clone)]
pub struct DatabaseInfo {
    pub version: String,
    pub database_name: String,
    pub current_user: String,
}
