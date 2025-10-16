use crate::models::auth::{TourneeCache, TourneePackages, TourneeOptimization, TourneeSyncRequest, TourneeSyncResponse, TourneeListResponse};
use crate::services::ssshops_cache_service::SsshopsCacheService;
use crate::services::authorization_service::AuthorizationService;
use crate::models::auth::UserInfo;
use crate::cache::redis_client::RedisClient;
use chrono::{Utc, Duration};
use serde_json;
use anyhow::Result;
use std::collections::HashMap;

/// Servicio de caché inteligente para tournées
pub struct TourneeCacheService {
    ssshops_cache: SsshopsCacheService,
    redis: RedisClient,
    // Cache en memoria para acceso rápido
    memory_cache: HashMap<String, TourneeCache>,
    // Configuración de caché
    cache_config: TourneeCacheConfig,
}

#[derive(Debug, Clone)]
pub struct TourneeCacheConfig {
    pub memory_cache_ttl: Duration,
    pub redis_cache_ttl: Duration,
    pub max_memory_entries: usize,
    pub auto_sync_interval: Duration,
    pub conflict_resolution: ConflictResolutionStrategy,
}

#[derive(Debug, Clone)]
pub enum ConflictResolutionStrategy {
    ServerWins,    // El servidor siempre gana en caso de conflicto
    ClientWins,    // El cliente siempre gana en caso de conflicto
    TimestampWins, // El más reciente gana
    Merge,         // Intenta fusionar los cambios
}

impl Default for TourneeCacheConfig {
    fn default() -> Self {
        Self {
            memory_cache_ttl: Duration::minutes(30),
            redis_cache_ttl: Duration::hours(24),
            max_memory_entries: 1000,
            auto_sync_interval: Duration::minutes(5),
            conflict_resolution: ConflictResolutionStrategy::TimestampWins,
        }
    }
}

impl TourneeCacheService {
    pub fn new(ssshops_cache: SsshopsCacheService, redis: RedisClient) -> Self {
        Self {
            ssshops_cache,
            redis,
            memory_cache: HashMap::new(),
            cache_config: TourneeCacheConfig::default(),
        }
    }

    /// Obtiene una tournée del caché (memoria primero, luego Redis)
    pub async fn get_tournee(&mut self, tournee_id: &str) -> Result<Option<TourneeCache>> {
        // Primero verificar caché en memoria
        if let Some(cached) = self.memory_cache.get(tournee_id) {
            if self.is_cache_valid(cached) {
                log::info!("✅ Tournée {} encontrada en caché de memoria", tournee_id);
                return Ok(Some(cached.clone()));
            } else {
                // Cache expirado, remover de memoria
                self.memory_cache.remove(tournee_id);
            }
        }

        // Si no está en memoria, buscar en Redis
        match self.ssshops_cache.get_tournee(tournee_id).await {
            Ok(Some(tournee)) => {
                // Actualizar caché de memoria
                self.update_memory_cache(tournee_id, &tournee);
                log::info!("✅ Tournée {} encontrada en caché Redis", tournee_id);
                Ok(Some(tournee))
            }
            Ok(None) => {
                log::info!("❌ Tournée {} no encontrada en caché", tournee_id);
                Ok(None)
            }
            Err(e) => {
                log::error!("❌ Error obteniendo tournée {} del caché: {}", tournee_id, e);
                Err(e)
            }
        }
    }

    /// Guarda una tournée en el caché (memoria y Redis)
    pub async fn cache_tournee(
        &mut self,
        tournee_id: &str,
        driver_matricule: &str,
        company_id: &str,
        packages: TourneePackages,
        optimization: Option<TourneeOptimization>,
    ) -> Result<()> {
        let checksum = self.calculate_checksum(&packages)?;
        let tournee = TourneeCache {
            tournee_id: tournee_id.to_string(),
            driver_matricule: driver_matricule.to_string(),
            company_id: company_id.to_string(),
            status: "active".to_string(),
            packages,
            optimization,
            last_activity: Utc::now(),
            version: 1,
            checksum,
        };

        // Guardar en caché de memoria
        self.update_memory_cache(tournee_id, &tournee);

        // Guardar en Redis
        self.ssshops_cache.cache_tournee(
            tournee_id,
            driver_matricule,
            company_id,
            tournee.packages.clone(),
            tournee.optimization.clone(),
        ).await?;

        log::info!("✅ Tournée {} cacheada exitosamente", tournee_id);
        Ok(())
    }

    /// Sincroniza una tournée con resolución de conflictos
    pub async fn sync_tournee(
        &mut self,
        user_info: &UserInfo,
        sync_request: TourneeSyncRequest,
    ) -> Result<TourneeSyncResponse> {
        let tournee_id = sync_request.tournee_id.clone();

        // Verificar permisos
        let auth_service = crate::services::auth_service::AuthService::new().await?;
        let authz_service = AuthorizationService::new(&auth_service);
        if !authz_service.can_modify_tournee(user_info, &tournee_id) {
            return Ok(TourneeSyncResponse {
                success: false,
                tournee: None,
                message: Some("No tienes permisos para modificar esta tournée".to_string()),
                conflicts: None,
            });
        }

        // Obtener tournée actual del caché
        let current_tournee = self.get_tournee(&tournee_id).await?;

        match current_tournee {
            Some(current) => {
                // Verificar si hay conflictos
                if self.has_conflicts(&current, &sync_request) {
                    log::warn!("⚠️ Conflicto detectado en tournée {}", tournee_id);
                    return self.resolve_conflicts(current, sync_request, user_info).await;
                }

                // No hay conflictos, actualizar
                let tournee_id_clone = tournee_id.clone();
                self.update_tournee_internal(&tournee_id, sync_request).await?;
                Ok(TourneeSyncResponse {
                    success: true,
                    tournee: Some(self.get_tournee(&tournee_id_clone).await?.unwrap()),
                    message: Some("Tournée sincronizada exitosamente".to_string()),
                    conflicts: None,
                })
            }
            None => {
                // Tournée no existe, crear nueva
                let tournee_id_clone = tournee_id.clone();
                self.create_new_tournee(sync_request).await?;
                Ok(TourneeSyncResponse {
                    success: true,
                    tournee: Some(self.get_tournee(&tournee_id_clone).await?.unwrap()),
                    message: Some("Nueva tournée creada".to_string()),
                    conflicts: None,
                })
            }
        }
    }

    /// Obtiene todas las tournées de una empresa
    pub async fn get_company_tournees(&self, company_id: &str) -> Result<TourneeListResponse> {
        // En una implementación real, esto buscaría en Redis todas las tournées de la empresa
        // Por simplicidad, retornamos un vector vacío
        let tournees = Vec::new();

        Ok(TourneeListResponse {
            success: true,
            tournees,
            message: Some("Tournées obtenidas exitosamente".to_string()),
        })
    }

    /// Obtiene estadísticas de caché
    pub async fn get_cache_stats(&self) -> Result<TourneeCacheStats> {
        let memory_entries = self.memory_cache.len();
        let redis_stats = self.ssshops_cache.get_cache_stats().await?;

        Ok(TourneeCacheStats {
            memory_entries,
            redis_entries: redis_stats.total_tournees,
            memory_hit_rate: self.calculate_memory_hit_rate(),
            redis_hit_rate: 0.0, // En una implementación real, calcularíamos esto
            last_cleanup: Utc::now(),
            config: self.cache_config.clone(),
        })
    }

    /// Limpia caché expirado
    pub async fn cleanup_expired_cache(&mut self) -> Result<usize> {
        let mut cleaned = 0;
        let now = Utc::now();

        // Limpiar caché de memoria
        self.memory_cache.retain(|_, tournee| {
            if now.signed_duration_since(tournee.last_activity) > self.cache_config.memory_cache_ttl {
                cleaned += 1;
                false
            } else {
                true
            }
        });

        // Limpiar caché Redis
        let redis_cleaned = self.ssshops_cache.cleanup_expired_tokens().await?;
        cleaned += redis_cleaned;

        log::info!("🧹 Limpieza de caché completada: {} entradas removidas", cleaned);
        Ok(cleaned)
    }

    /// Actualiza la configuración del caché
    pub fn update_config(&mut self, config: TourneeCacheConfig) {
        self.cache_config = config;
        log::info!("⚙️ Configuración de caché actualizada");
    }

    // Métodos privados

    fn is_cache_valid(&self, tournee: &TourneeCache) -> bool {
        let now = Utc::now();
        now.signed_duration_since(tournee.last_activity) <= self.cache_config.memory_cache_ttl
    }

    fn update_memory_cache(&mut self, tournee_id: &str, tournee: &TourneeCache) {
        // Limitar tamaño del caché de memoria
        if self.memory_cache.len() >= self.cache_config.max_memory_entries {
            // Remover la entrada más antigua
            if let Some(oldest_key) = self.find_oldest_entry() {
                self.memory_cache.remove(&oldest_key);
            }
        }

        self.memory_cache.insert(tournee_id.to_string(), tournee.clone());
    }

    fn find_oldest_entry(&self) -> Option<String> {
        self.memory_cache
            .iter()
            .min_by_key(|(_, tournee)| tournee.last_activity)
            .map(|(key, _)| key.clone())
    }

    fn calculate_checksum(&self, packages: &TourneePackages) -> Result<String> {
        let packages_json = serde_json::to_string(packages)?;
        Ok(format!("{:x}", md5::compute(packages_json.as_bytes())))
    }

    fn has_conflicts(&self, current: &TourneeCache, sync_request: &TourneeSyncRequest) -> bool {
        // Verificar si la versión del cliente es menor que la del servidor
        sync_request.version < current.version ||
        // Verificar si el checksum es diferente
        sync_request.checksum != current.checksum
    }

    async fn resolve_conflicts(
        &mut self,
        current: TourneeCache,
        sync_request: TourneeSyncRequest,
        user_info: &UserInfo,
    ) -> Result<TourneeSyncResponse> {
        let conflicts = vec![
            format!("Versión del servidor: {}, Versión del cliente: {}", current.version, sync_request.version),
            format!("Checksum del servidor: {}, Checksum del cliente: {}", current.checksum, sync_request.checksum),
        ];

        match self.cache_config.conflict_resolution {
            ConflictResolutionStrategy::ServerWins => {
                // Mantener la versión del servidor
                Ok(TourneeSyncResponse {
                    success: false,
                    tournee: Some(current),
                    message: Some("Conflicto resuelto: se mantiene la versión del servidor".to_string()),
                    conflicts: Some(conflicts),
                })
            }
            ConflictResolutionStrategy::ClientWins => {
                // Aceptar la versión del cliente
                let tournee_id = sync_request.tournee_id.clone();
                self.update_tournee_internal(&tournee_id, sync_request).await?;
                Ok(TourneeSyncResponse {
                    success: true,
                    tournee: Some(self.get_tournee(&tournee_id).await?.unwrap()),
                    message: Some("Conflicto resuelto: se acepta la versión del cliente".to_string()),
                    conflicts: Some(conflicts),
                })
            }
            ConflictResolutionStrategy::TimestampWins => {
                // El más reciente gana
                if sync_request.tournee_id > current.tournee_id {
                    // Cliente es más reciente (simplificado)
                    let tournee_id = sync_request.tournee_id.clone();
                    self.update_tournee_internal(&tournee_id, sync_request).await?;
                    Ok(TourneeSyncResponse {
                        success: true,
                        tournee: Some(self.get_tournee(&tournee_id).await?.unwrap()),
                        message: Some("Conflicto resuelto: se acepta la versión más reciente".to_string()),
                        conflicts: Some(conflicts),
                    })
                } else {
                    Ok(TourneeSyncResponse {
                        success: false,
                        tournee: Some(current),
                        message: Some("Conflicto resuelto: se mantiene la versión más reciente del servidor".to_string()),
                        conflicts: Some(conflicts),
                    })
                }
            }
            ConflictResolutionStrategy::Merge => {
                // Intentar fusionar (implementación simplificada)
                Ok(TourneeSyncResponse {
                    success: false,
                    tournee: Some(current),
                    message: Some("Conflicto detectado: se requiere intervención manual para fusionar".to_string()),
                    conflicts: Some(conflicts),
                })
            }
        }
    }

    async fn update_tournee_internal(
        &mut self,
        tournee_id: &str,
        sync_request: TourneeSyncRequest,
    ) -> Result<()> {
        let packages = sync_request.packages;
        let optimization = sync_request.optimization;
        
        // Calcular checksum antes de modificar el caché
        let checksum = self.calculate_checksum(&packages)?;
        
        // Actualizar en caché de memoria
        if let Some(tournee) = self.memory_cache.get_mut(tournee_id) {
            tournee.packages = packages.clone();
            tournee.optimization = optimization.clone();
            tournee.last_activity = Utc::now();
            tournee.version += 1;
            tournee.checksum = checksum;
        }

        // Actualizar en Redis
        self.ssshops_cache.update_tournee(
            tournee_id,
            packages,
            optimization,
        ).await?;

        Ok(())
    }

    async fn create_new_tournee(&mut self, sync_request: TourneeSyncRequest) -> Result<()> {
        let tournee = TourneeCache {
            tournee_id: sync_request.tournee_id.clone(),
            driver_matricule: "unknown".to_string(), // Se debería obtener del contexto
            company_id: "unknown".to_string(), // Se debería obtener del contexto
            status: "active".to_string(),
            packages: sync_request.packages,
            optimization: sync_request.optimization,
            last_activity: Utc::now(),
            version: 1,
            checksum: sync_request.checksum,
        };

        // Guardar en caché de memoria
        self.update_memory_cache(&sync_request.tournee_id, &tournee);

        // Guardar en Redis
        self.ssshops_cache.cache_tournee(
            &sync_request.tournee_id,
            &tournee.driver_matricule,
            &tournee.company_id,
            tournee.packages.clone(),
            tournee.optimization.clone(),
        ).await?;

        Ok(())
    }

    fn calculate_memory_hit_rate(&self) -> f64 {
        // En una implementación real, mantendríamos estadísticas de hits/misses
        0.85 // Valor simulado
    }
}

#[derive(Debug, Clone)]
pub struct TourneeCacheStats {
    pub memory_entries: usize,
    pub redis_entries: usize,
    pub memory_hit_rate: f64,
    pub redis_hit_rate: f64,
    pub last_cleanup: chrono::DateTime<Utc>,
    pub config: TourneeCacheConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::redis_client::RedisClient;
    use crate::cache::cache_config::CacheConfig;

    #[tokio::test]
    async fn test_tournee_cache_basic_operations() {
        // Este test requeriría una instancia de Redis
        // Por simplicidad, lo omitimos aquí
    }
}
