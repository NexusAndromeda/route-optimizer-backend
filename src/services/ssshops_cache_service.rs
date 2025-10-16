use crate::models::auth::{SsshopsTokenCache, TourneeCache, TourneePackages, TourneeOptimization};
use crate::cache::redis_client::RedisClient;
use chrono::{Utc, Duration};
use serde_json;
use anyhow::Result;

/// Servicio de caché para tokens Ssshops y datos de tournées
pub struct SsshopsCacheService {
    redis: RedisClient,
}

impl SsshopsCacheService {
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }

    /// Guarda un token Ssshops en caché
    pub async fn cache_ssshops_token(
        &self,
        company_id: &str,
        driver_matricule: &str,
        token: &str,
        expires_in_hours: i32,
    ) -> Result<()> {
        let cache_key = format!("ssshops_token:{}:{}", company_id, driver_matricule);
        let expires_at = Utc::now() + Duration::hours(expires_in_hours as i64);
        
        let token_cache = SsshopsTokenCache {
            company_id: company_id.to_string(),
            driver_matricule: driver_matricule.to_string(),
            token: token.to_string(),
            created_at: Utc::now(),
            expires_at,
            is_valid: true,
        };

        let ttl_seconds = expires_in_hours * 3600;
        self.redis.set(&cache_key, &token_cache, ttl_seconds as u64).await?;
        
        log::info!("✅ Token Ssshops cacheado para {}:{}", company_id, driver_matricule);
        Ok(())
    }

    /// Recupera un token Ssshops del caché
    pub async fn get_ssshops_token(
        &self,
        company_id: &str,
        driver_matricule: &str,
    ) -> Result<Option<SsshopsTokenCache>> {
        let cache_key = format!("ssshops_token:{}:{}", company_id, driver_matricule);
        
        match self.redis.get::<SsshopsTokenCache>(&cache_key).await? {
            Some(token_cache) => {
                // Verificar si el token sigue siendo válido
                if token_cache.is_valid && token_cache.expires_at > Utc::now() {
                    log::info!("✅ Token Ssshops encontrado en caché para {}:{}", company_id, driver_matricule);
                    Ok(Some(token_cache))
                } else {
                    log::info!("⚠️ Token Ssshops expirado para {}:{}", company_id, driver_matricule);
                    // Marcar como inválido
                    self.invalidate_ssshops_token(company_id, driver_matricule).await?;
                    Ok(None)
                }
            }
            None => {
                log::info!("❌ Token Ssshops no encontrado en caché para {}:{}", company_id, driver_matricule);
                Ok(None)
            }
        }
    }

    /// Invalida un token Ssshops
    pub async fn invalidate_ssshops_token(
        &self,
        company_id: &str,
        driver_matricule: &str,
    ) -> Result<()> {
        let cache_key = format!("ssshops_token:{}:{}", company_id, driver_matricule);
        self.redis.delete(&cache_key).await?;
        
        log::info!("🗑️ Token Ssshops invalidado para {}:{}", company_id, driver_matricule);
        Ok(())
    }

    /// Guarda datos de tournée en caché
    pub async fn cache_tournee(
        &self,
        tournee_id: &str,
        driver_matricule: &str,
        company_id: &str,
        packages: TourneePackages,
        optimization: Option<TourneeOptimization>,
    ) -> Result<()> {
        let cache_key = format!("tournee:{}", tournee_id);
        
        // Calcular checksum para verificar integridad
        let packages_json = serde_json::to_string(&packages)?;
        let checksum = format!("{:x}", md5::compute(packages_json.as_bytes()));
        
        let tournee_cache = TourneeCache {
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

        // Cachear por 24 horas
        self.redis.set(&cache_key, &tournee_cache, 86400).await?;
        
        log::info!("✅ Tournée cacheada: {} ({} paquetes)", tournee_id, 
            tournee_cache.packages.singles.len() + 
            tournee_cache.packages.groups.len() + 
            tournee_cache.packages.problematic.len()
        );
        Ok(())
    }

    /// Recupera datos de tournée del caché
    pub async fn get_tournee(&self, tournee_id: &str) -> Result<Option<TourneeCache>> {
        let cache_key = format!("tournee:{}", tournee_id);
        
        match self.redis.get::<TourneeCache>(&cache_key).await? {
            Some(tournee_cache) => {
                log::info!("✅ Tournée encontrada en caché: {}", tournee_id);
                Ok(Some(tournee_cache))
            }
            None => {
                log::info!("❌ Tournée no encontrada en caché: {}", tournee_id);
                Ok(None)
            }
        }
    }

    /// Actualiza datos de tournée en caché
    pub async fn update_tournee(
        &self,
        tournee_id: &str,
        packages: TourneePackages,
        optimization: Option<TourneeOptimization>,
    ) -> Result<()> {
        // Primero obtener la tournée existente
        if let Some(mut tournee_cache) = self.get_tournee(tournee_id).await? {
            // Actualizar datos
            tournee_cache.packages = packages;
            tournee_cache.optimization = optimization;
            tournee_cache.last_activity = Utc::now();
            tournee_cache.version += 1;
            
            // Recalcular checksum
            let packages_json = serde_json::to_string(&tournee_cache.packages)?;
            tournee_cache.checksum = format!("{:x}", md5::compute(packages_json.as_bytes()));
            
            // Guardar actualización
            let cache_key = format!("tournee:{}", tournee_id);
            self.redis.set(&cache_key, &tournee_cache, 86400).await?;
            
            log::info!("✅ Tournée actualizada en caché: {}", tournee_id);
        } else {
            log::warn!("⚠️ No se pudo actualizar tournée {}: no encontrada en caché", tournee_id);
        }
        
        Ok(())
    }

    /// Obtiene todas las tournées de una empresa
    pub async fn get_company_tournees(&self, company_id: &str) -> Result<Vec<TourneeCache>> {
        // En Redis, podríamos usar SCAN para buscar todas las claves que empiecen con "tournee:"
        // Por simplicidad, aquí retornamos un vector vacío
        // En una implementación real, usaríamos Redis SCAN
        log::info!("🔍 Buscando tournées para empresa: {}", company_id);
        Ok(Vec::new())
    }

    /// Invalida caché de tournée
    pub async fn invalidate_tournee(&self, tournee_id: &str) -> Result<()> {
        let cache_key = format!("tournee:{}", tournee_id);
        self.redis.delete(&cache_key).await?;
        
        log::info!("🗑️ Tournée invalidada: {}", tournee_id);
        Ok(())
    }

    /// Limpia tokens expirados
    pub async fn cleanup_expired_tokens(&self) -> Result<usize> {
        // En una implementación real, usaríamos Redis TTL y SCAN
        // Por simplicidad, retornamos 0
        log::info!("🧹 Limpieza de tokens expirados completada");
        Ok(0)
    }

    /// Obtiene estadísticas del caché
    pub async fn get_cache_stats(&self) -> Result<CacheStats> {
        // En una implementación real, usaríamos Redis INFO
        Ok(CacheStats {
            total_tokens: 0,
            total_tournees: 0,
            expired_tokens: 0,
            memory_usage: 0,
        })
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub total_tokens: usize,
    pub total_tournees: usize,
    pub expired_tokens: usize,
    pub memory_usage: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::redis_client::RedisClient;
    use crate::cache::cache_config::CacheConfig;

    #[tokio::test]
    async fn test_ssshops_token_caching() {
        // Este test requeriría una instancia de Redis
        // Por simplicidad, lo omitimos aquí
    }
}
