use anyhow::Result;
use redis::{aio::ConnectionManager, AsyncCommands, RedisResult};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, error, info, warn};

use super::{CacheConfig, CacheOperations};

/// Cliente Redis con connection pooling y operaciones async
#[derive(Clone)]
pub struct RedisClient {
    manager: ConnectionManager,
    config: CacheConfig,
}

impl RedisClient {
    /// Crear nuevo cliente Redis
    pub async fn new(config: CacheConfig) -> Result<Self> {
        info!("🔗 Conectando a Redis: {}", config.redis_url);
        
        let client = redis::Client::open(config.redis_url.clone())?;
        let manager = ConnectionManager::new(client).await?;
        
        // Test de conexión usando un comando simple
        let mut conn = manager.clone();
        let _: () = redis::cmd("PING").query_async(&mut conn).await?;
        
        info!("✅ Redis conectado exitosamente");
        
        Ok(Self { manager, config })
    }
    
    /// Generar clave de cache con prefijo
    fn make_key(&self, prefix: &str, identifier: &str) -> String {
        format!("delivery_optimizer:{}:{}", prefix, identifier)
    }
    
    /// Generar clave de auth cache
    pub fn auth_key(&self, username: &str, societe: &str) -> String {
        self.make_key("auth", &format!("{}:{}", username, societe))
    }
    
    /// Generar clave de tournée cache
    pub fn tournee_key(&self, societe: &str, matricule: &str, date: &str) -> String {
        self.make_key("tournee", &format!("{}:{}:{}", societe, matricule, date))
    }
    
    /// Generar clave de rate limiting
    pub fn rate_limit_key(&self, identifier: &str) -> String {
        self.make_key("rate_limit", identifier)
    }
}

#[async_trait::async_trait]
impl CacheOperations for RedisClient {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.manager.clone();
        
        match conn.get::<_, Option<String>>(key).await {
            Ok(Some(value)) => {
                debug!("📥 Cache HIT para clave: {}", key);
                let deserialized: T = serde_json::from_str(&value)?;
                Ok(Some(deserialized))
            }
            Ok(None) => {
                debug!("❌ Cache MISS para clave: {}", key);
                Ok(None)
            }
            Err(e) => {
                warn!("⚠️ Error leyendo cache para clave {}: {}", key, e);
                Ok(None)
            }
        }
    }
    
    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T, ttl: u64) -> Result<()> {
        let mut conn = self.manager.clone();
        
        let serialized = serde_json::to_string(value)?;
        
        let result: RedisResult<()> = conn.set_ex(key, serialized, ttl).await;
        
        match result {
            Ok(()) => {
                debug!("💾 Cache SET para clave: {} (TTL: {}s)", key, ttl);
                Ok(())
            }
            Err(e) => {
                error!("❌ Error guardando en cache para clave {}: {}", key, e);
                Err(anyhow::anyhow!("Error de Redis: {}", e))
            }
        }
    }
    
    async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.manager.clone();
        
        let result: RedisResult<i64> = conn.del(key).await;
        
        match result {
            Ok(count) => {
                debug!("🗑️ Cache DELETE para clave: {} (eliminados: {})", key, count);
                Ok(())
            }
            Err(e) => {
                warn!("⚠️ Error eliminando cache para clave {}: {}", key, e);
                Ok(()) // No fallar si no se puede eliminar
            }
        }
    }
    
    async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.manager.clone();
        
        match conn.exists(key).await {
            Ok(exists) => {
                debug!("🔍 Cache EXISTS para clave {}: {}", key, exists);
                Ok(exists)
            }
            Err(e) => {
                warn!("⚠️ Error verificando existencia de clave {}: {}", key, e);
                Ok(false)
            }
        }
    }
    
    async fn ttl(&self, key: &str) -> Result<Option<u64>> {
        let mut conn = self.manager.clone();
        
        match conn.ttl(key).await {
            Ok(ttl) => {
                if ttl > 0 {
                    debug!("⏰ Cache TTL para clave {}: {}s", key, ttl);
                    Ok(Some(ttl))
                } else {
                    debug!("⏰ Cache TTL para clave {}: expirado", key);
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("⚠️ Error obteniendo TTL para clave {}: {}", key, e);
                Ok(None)
            }
        }
    }
}

impl RedisClient {
    /// Obtener estadísticas del cache
    pub async fn get_stats(&self) -> Result<CacheStats> {
        let mut conn = self.manager.clone();
        
        let info: String = redis::cmd("INFO").query_async(&mut conn).await?;
        
        // Parsear información básica de Redis
        let mut stats = CacheStats::default();
        
        for line in info.lines() {
            if line.starts_with("connected_clients:") {
                if let Some(count) = line.split(':').nth(1) {
                    stats.connected_clients = count.parse().unwrap_or(0);
                }
            } else if line.starts_with("used_memory_human:") {
                if let Some(memory) = line.split(':').nth(1) {
                    stats.used_memory = memory.to_string();
                }
            } else if line.starts_with("total_commands_processed:") {
                if let Some(count) = line.split(':').nth(1) {
                    stats.total_commands = count.parse().unwrap_or(0);
                }
            }
        }
        
        Ok(stats)
    }
    
    /// Limpiar cache completo (¡CUIDADO!)
    pub async fn flush_all(&self) -> Result<()> {
        let mut conn = self.manager.clone();
        
        info!("🧹 Limpiando cache completo...");
        let _: () = redis::cmd("FLUSHALL").query_async(&mut conn).await?;
        
        info!("✅ Cache limpiado completamente");
        Ok(())
    }
    
    /// Verificar si Redis está conectado
    pub async fn is_connected(&self) -> bool {
        let mut conn = self.manager.clone();
        match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
            Ok(response) => response == "PONG",
            Err(_) => false,
        }
    }
}

/// Estadísticas del cache
#[derive(Debug, Default)]
pub struct CacheStats {
    pub connected_clients: u32,
    pub used_memory: String,
    pub total_commands: u64,
}
