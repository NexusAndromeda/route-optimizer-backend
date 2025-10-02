//! Configuración de cache
//! 
//! Este módulo contiene la configuración para el sistema de cache.

use serde::{Deserialize, Serialize};

/// Configuración del cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub redis_url: String,
    pub default_ttl: u64,
    pub max_connections: u32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            default_ttl: 3600, // 1 hora
            max_connections: 10,
        }
    }
}

/// Operaciones de cache
#[derive(Debug, Clone)]
pub enum CacheOperations {
    Get,
    Set,
    Delete,
    Exists,
    Flush,
}
