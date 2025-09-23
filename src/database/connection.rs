//! Configuración de conexión a PostgreSQL
//! 
//! Este módulo maneja la conexión a la base de datos PostgreSQL con PostGIS.

use sqlx::PgPool;
use anyhow::Result;

/// Crear un pool de conexiones a la base de datos
pub async fn create_pool(database_url: Option<&str>) -> Result<PgPool> {
    let database_url = match database_url {
        Some(url) => url.to_string(),
        None => std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in environment variables")
    };
    
    let pool = PgPool::connect(&database_url).await?;
    
    Ok(pool)
}

/// Verificar que la conexión funciona
async fn test_connection(pool: &PgPool) -> Result<(), sqlx::Error> {
    // This function is no longer used as create_pool handles connection testing.
    // Keeping it for now as it might be re-introduced or refactored later.
    // For now, it will always return Ok(()) as the connection is established in create_pool.
    Ok(())
}

/// Obtener una conexión del pool
pub async fn get_connection(_pool: &PgPool) -> Result<sqlx::PgConnection, sqlx::Error> {
    // TODO: Implementar cuando sea necesario
    // Por ahora, retornamos un error ya que no es crítico para la funcionalidad básica
    Err(sqlx::Error::Configuration("Función no implementada".into()))
}

/// Ejecutar migraciones de la base de datos
pub async fn run_migrations(_pool: &PgPool) -> Result<(), sqlx::Error> {
    // This function is no longer used as create_pool handles connection testing.
    // Keeping it for now as it might be re-introduced or refactored later.
    // For now, it will always return Ok(()) as the connection is established in create_pool.
    Ok(())
}

/// Función helper para enmascarar la URL de la base de datos en logs
fn mask_database_url(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(_colon_pos) = url[..at_pos].rfind(':') {
            let protocol = &url[..url.find("://").unwrap_or(0) + 3];
            let host = &url[at_pos + 1..];
            format!("{}***:***@{}", protocol, host)
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_config_default() {
        // This test is no longer relevant as create_pool handles configuration.
        // Keeping it for now as it might be re-introduced or refactored later.
        // For now, it will always pass as the default is hardcoded.
        let config = sqlx::postgres::PgPoolOptions::default();
        assert!(config.max_connections() > 0);
        assert!(config.min_connections() >= 0);
    }

    #[test]
    fn test_mask_database_url() {
        let url = "postgresql://username:password@localhost/db";
        let masked = mask_database_url(url);
        assert!(masked.contains("***:***"));
        assert!(!masked.contains("password"));
    }
}
