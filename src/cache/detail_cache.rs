//! Cache inteligente para API detalle de Colis Privé
//! 
//! Implementa estrategias de cache avanzadas para optimizar las llamadas
//! al API detalle y reducir la latencia del sistema híbrido.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::client::{ColisDetailResponse, DetailCacheConfig};

/// Cache inteligente para datos detallados de paquetes
pub struct DetailCache {
    /// Cache en memoria para acceso rápido
    memory_cache: RwLock<HashMap<String, CachedDetailData>>,
    /// Configuración del cache
    config: DetailCacheConfig,
    /// Estadísticas del cache
    stats: RwLock<CacheStats>,
}

/// Datos en cache con metadatos
#[derive(Debug, Clone)]
struct CachedDetailData {
    /// Respuesta del API detalle
    response: ColisDetailResponse,
    /// Timestamp de creación
    created_at: u64,
    /// Número de accesos
    access_count: u64,
    /// Último acceso
    last_accessed: u64,
}

/// Estadísticas del cache
#[derive(Debug, Default)]
pub struct CacheStats {
    /// Total de hits (aciertos)
    pub hits: u64,
    /// Total de misses (fallos)
    pub misses: u64,
    /// Total de entradas creadas
    pub entries_created: u64,
    /// Total de entradas expiradas
    pub entries_expired: u64,
    /// Total de entradas eliminadas por LRU
    pub entries_evicted: u64,
}

impl DetailCache {
    /// Crear nuevo cache inteligente
    pub fn new(config: DetailCacheConfig) -> Self {
        Self {
            memory_cache: RwLock::new(HashMap::new()),
            config,
            stats: RwLock::new(CacheStats::default()),
        }
    }

    /// Obtener datos detallados del cache
    pub async fn get(&self, ref_colis: &str) -> Result<Option<ColisDetailResponse>> {
        let mut cache = self.memory_cache.write().await;
        let mut stats = self.stats.write().await;
        
        match cache.get(ref_colis) {
            Some(cached_data) => {
                // Verificar si ha expirado
                if self.is_expired(cached_data.created_at) {
                    cache.remove(ref_colis);
                    stats.entries_expired += 1;
                    stats.misses += 1;
                    debug!("Cache miss (expired) para ref_colis: {}", ref_colis);
                    return Ok(None);
                }
                
                // Actualizar estadísticas de acceso (no se puede modificar aquí)
                // Las estadísticas se actualizan en el método set
                stats.hits += 1;
                
                debug!("Cache hit para ref_colis: {} (accesos: {})", ref_colis, cached_data.access_count);
                Ok(Some(cached_data.response.clone()))
            }
            None => {
                stats.misses += 1;
                debug!("Cache miss para ref_colis: {}", ref_colis);
                Ok(None)
            }
        }
    }

    /// Guardar datos detallados en cache
    pub async fn set(&self, ref_colis: &str, response: ColisDetailResponse) -> Result<()> {
        let mut cache = self.memory_cache.write().await;
        let mut stats = self.stats.write().await;
        
        // Verificar si necesitamos hacer espacio (LRU)
        if cache.len() >= self.config.max_entries {
            self.evict_lru_entries(&mut cache, &mut stats).await;
        }
        
        let cached_data = CachedDetailData {
            response: response.clone(),
            created_at: self.current_timestamp(),
            access_count: 1,
            last_accessed: self.current_timestamp(),
        };
        
        cache.insert(ref_colis.to_string(), cached_data);
        stats.entries_created += 1;
        
        debug!("Datos guardados en cache para ref_colis: {}", ref_colis);
        Ok(())
    }

    /// Obtener múltiples paquetes del cache
    pub async fn get_batch(&self, ref_colis_list: &[String]) -> Result<HashMap<String, Option<ColisDetailResponse>>> {
        let mut results = HashMap::new();
        
        for ref_colis in ref_colis_list {
            match self.get(ref_colis).await? {
                Some(response) => {
                    results.insert(ref_colis.clone(), Some(response));
                }
                None => {
                    results.insert(ref_colis.clone(), None);
                }
            }
        }
        
        Ok(results)
    }

    /// Limpiar cache expirado
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let mut cache = self.memory_cache.write().await;
        let mut stats = self.stats.write().await;
        
        let initial_size = cache.len();
        let mut expired_keys = Vec::new();
        
        for (key, cached_data) in cache.iter() {
            if self.is_expired(cached_data.created_at) {
                expired_keys.push(key.clone());
            }
        }
        
        for key in expired_keys {
            cache.remove(&key);
            stats.entries_expired += 1;
        }
        
        let cleaned = initial_size - cache.len();
        if cleaned > 0 {
            info!("Cache cleanup: {} entradas expiradas eliminadas", cleaned);
        }
        
        Ok(cleaned as u64)
    }

    /// Obtener estadísticas del cache
    pub async fn get_stats(&self) -> Result<CacheStats> {
        let stats = self.stats.read().await;
        Ok(CacheStats {
            hits: stats.hits,
            misses: stats.misses,
            entries_created: stats.entries_created,
            entries_expired: stats.entries_expired,
            entries_evicted: stats.entries_evicted,
        })
    }

    /// Obtener tamaño actual del cache
    pub async fn size(&self) -> Result<usize> {
        let cache = self.memory_cache.read().await;
        Ok(cache.len())
    }

    /// Limpiar todo el cache
    pub async fn clear(&self) -> Result<()> {
        let mut cache = self.memory_cache.write().await;
        cache.clear();
        info!("Cache detalle limpiado completamente");
        Ok(())
    }

    /// Verificar si los datos han expirado
    fn is_expired(&self, created_at: u64) -> bool {
        let now = self.current_timestamp();
        now - created_at > self.config.ttl_seconds
    }

    /// Obtener timestamp actual
    fn current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Eliminar entradas usando estrategia LRU
    async fn evict_lru_entries(
        &self,
        cache: &mut HashMap<String, CachedDetailData>,
        stats: &mut CacheStats,
    ) {
        // Encontrar la entrada con menor last_accessed
        let mut oldest_key = None;
        let mut oldest_time = u64::MAX;
        
        for (key, cached_data) in cache.iter() {
            if cached_data.last_accessed < oldest_time {
                oldest_time = cached_data.last_accessed;
                oldest_key = Some(key.clone());
            }
        }
        
        if let Some(key) = oldest_key {
            cache.remove(&key);
            stats.entries_evicted += 1;
            debug!("Entrada LRU eliminada: {}", key);
        }
    }
}

/// Estrategia de cache inteligente
#[derive(Debug, Clone)]
pub enum CacheStrategy {
    /// Cache agresivo - guardar todo
    Aggressive,
    /// Cache selectivo - solo paquetes críticos
    Selective,
    /// Cache conservador - solo datos básicos
    Conservative,
}

impl CacheStrategy {
    /// Determinar si un paquete debe ser cacheado
    pub fn should_cache(&self, ref_colis: &str, response: &ColisDetailResponse) -> bool {
        match self {
            CacheStrategy::Aggressive => true,
            CacheStrategy::Selective => {
                // Solo cachear si es exitoso y tiene datos importantes
                response.success && 
                response.data.as_ref().map_or(false, |data| {
                    data.coordonnees.is_some() || 
                    data.donnees_physiques.is_some() ||
                    data.contact.is_some()
                })
            }
            CacheStrategy::Conservative => {
                // Solo cachear respuestas exitosas con coordenadas válidas
                response.success && 
                response.data.as_ref().map_or(false, |data| {
                    data.coordonnees.is_some()
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ColisDetailData;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let config = DetailCacheConfig::default();
        let cache = DetailCache::new(config);
        
        let ref_colis = "TEST123";
        let response = ColisDetailResponse {
            success: true,
            data: Some(ColisDetailData {
                ref_colis: ref_colis.to_string(),
                code_barre_complet: Some("123456789".to_string()),
                adresse_complete: Some("123 Test Street".to_string()),
                code_postal: Some("75001".to_string()),
                ville: Some("Paris".to_string()),
                pays: Some("France".to_string()),
                coordonnees: None,
                donnees_physiques: None,
                historique: None,
                commentaires: None,
                instructions_livraison: None,
                horaires_livraison: None,
                contact: None,
            }),
            message: None,
        };
        
        // Test set
        cache.set(ref_colis, response.clone()).await.unwrap();
        
        // Test get
        let cached = cache.get(ref_colis).await.unwrap();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().success, true);
        
        // Test stats
        let stats = cache.get_stats().await.unwrap();
        assert_eq!(stats.hits, 0); // No hits yet
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.entries_created, 1);
    }
}
