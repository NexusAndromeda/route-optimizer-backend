//! Sistema de cache Redis para optimizar performance y camuflar requests

pub mod redis_client;
pub mod auth_cache;
pub mod tournee_cache;
pub mod detail_cache;

pub use redis_client::RedisClient;
// auth_cache, tournee_cache - no se usan actualmente
pub use detail_cache::{DetailCache, CacheStrategy};

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

/// Trait para operaciones de cache genéricas
#[async_trait::async_trait]
pub trait CacheOperations {
    /// Obtener valor del cache
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    
    /// Guardar valor en cache
    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T, ttl: u64) -> Result<()>;
    
    /// Eliminar valor del cache
    async fn delete(&self, key: &str) -> Result<()>;
    
    /// Verificar si existe una clave
    async fn exists(&self, key: &str) -> Result<bool>;
    
    /// Obtener TTL de una clave
    async fn ttl(&self, key: &str) -> Result<Option<u64>>;
}

/// Configuración del cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub redis_url: String,
    pub default_ttl: u64,
    pub auth_cache_ttl: u64,
    pub tournee_cache_ttl: u64,
    pub max_connections: u32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            default_ttl: 3600,        // 1 hora por defecto
            auth_cache_ttl: 1800,     // 30 minutos para auth
            tournee_cache_ttl: 900,   // 15 minutos para tournée
            max_connections: 10,
        }
    }
}
