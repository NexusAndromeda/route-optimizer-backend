//! API endpoints para el sistema híbrido de optimización de rutas
//! 
//! Endpoints que combinan datos básicos con API detalle para optimizar entregas

use anyhow::Result;
use axum::{
    extract::State,
    response::Json,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::services::hybrid_processor::{HybridProcessor, HybridProcessingResult};
use crate::cache::CacheStrategy;
use crate::state::AppState;

/// Request para procesamiento híbrido
#[derive(Debug, Deserialize)]
pub struct HybridProcessingRequest {
    /// Lista de referencias de paquetes a procesar
    pub ref_colis_list: Vec<String>,
    /// Estrategia de cache a usar
    pub cache_strategy: Option<String>,
    /// Token SSO para autenticación
    pub sso_token: String,
}

/// Response del procesamiento híbrido
#[derive(Debug, Serialize)]
pub struct HybridProcessingResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<HybridProcessingResult>,
    pub processing_time_ms: u64,
}

/// Request para obtener datos detallados de un paquete
#[derive(Debug, Deserialize)]
pub struct PackageDetailRequest {
    pub ref_colis: String,
    pub sso_token: String,
}

/// Response de datos detallados
#[derive(Debug, Serialize)]
pub struct PackageDetailResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<crate::clients::ColisDetailResponse>,
}

/// Request para limpiar cache
#[derive(Debug, Deserialize)]
pub struct CacheCleanupRequest {
    pub cache_type: String, // "detail", "all"
}

/// Response de limpieza de cache
#[derive(Debug, Serialize)]
pub struct CacheCleanupResponse {
    pub success: bool,
    pub message: String,
    pub entries_cleaned: u64,
}

/// Crear router para endpoints híbridos
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/api/hybrid/process", post(process_packages_hybrid))
        .route("/api/hybrid/package-detail", post(get_package_detail))
        .route("/api/hybrid/cache/cleanup", post(cleanup_cache))
        .route("/api/hybrid/cache/stats", post(get_cache_stats))
}

/// Procesar paquetes con estrategia híbrida
pub async fn process_packages_hybrid(
    State(_state): State<AppState>,
    Json(request): Json<HybridProcessingRequest>,
) -> Result<Json<HybridProcessingResponse>, Json<HybridProcessingResponse>> {
    let start_time = std::time::Instant::now();
    
    info!("Iniciando procesamiento híbrido para {} paquetes", request.ref_colis_list.len());
    
    // Determinar estrategia de cache
    let cache_strategy = match request.cache_strategy.as_deref() {
        Some("aggressive") => CacheStrategy::Aggressive,
        Some("selective") => CacheStrategy::Selective,
        Some("conservative") => CacheStrategy::Conservative,
        _ => CacheStrategy::Selective, // Default
    };
    
    // Crear procesador híbrido
    let processor = match HybridProcessor::new(cache_strategy, &_state.config) {
        Ok(p) => p,
        Err(e) => {
            warn!("Error creando procesador híbrido: {}", e);
            return Ok(Json(HybridProcessingResponse {
                success: false,
                message: format!("Error inicializando procesador: {}", e),
                data: None,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
            }));
        }
    };
    
    // TODO: Obtener paquetes básicos de la base de datos
    // Por ahora, crear paquetes mock para testing
    let mock_packages = create_mock_packages(&request.ref_colis_list);
    
    // Procesar paquetes
    match processor.process_packages(mock_packages, &request.sso_token).await {
        Ok(result) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            info!("Procesamiento híbrido completado en {}ms", processing_time);
            
            Ok(Json(HybridProcessingResponse {
                success: true,
                message: "Procesamiento híbrido completado exitosamente".to_string(),
                data: Some(result),
                processing_time_ms: processing_time,
            }))
        }
        Err(e) => {
            warn!("Error en procesamiento híbrido: {}", e);
            Ok(Json(HybridProcessingResponse {
                success: false,
                message: format!("Error en procesamiento: {}", e),
                data: None,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
            }))
        }
    }
}

/// Obtener datos detallados de un paquete específico
pub async fn get_package_detail(
    State(_state): State<AppState>,
    Json(request): Json<PackageDetailRequest>,
) -> Result<Json<PackageDetailResponse>, Json<PackageDetailResponse>> {
    info!("Obteniendo datos detallados para paquete: {}", request.ref_colis);
    
    // Crear cliente
    let client = match crate::clients::ColisPriveWebClient::new(
        _state.config.colis_prive_auth_url.clone(),
        _state.config.colis_prive_tournee_url.clone(),
        _state.config.colis_prive_detail_url.clone(),
    ) {
        Ok(c) => c,
        Err(e) => {
            warn!("Error creando cliente: {}", e);
            return Ok(Json(PackageDetailResponse {
                success: false,
                message: format!("Error creando cliente: {}", e),
                data: None,
            }));
        }
    };
    
    // Obtener datos detallados
    match client.get_package_detail(&request.ref_colis, &request.sso_token).await {
        Ok(response) => {
            info!("Datos detallados obtenidos para paquete: {}", request.ref_colis);
            Ok(Json(PackageDetailResponse {
                success: true,
                message: "Datos detallados obtenidos exitosamente".to_string(),
                data: Some(response),
            }))
        }
        Err(e) => {
            warn!("Error obteniendo datos detallados: {}", e);
            Ok(Json(PackageDetailResponse {
                success: false,
                message: format!("Error obteniendo datos: {}", e),
                data: None,
            }))
        }
    }
}

/// Limpiar cache
pub async fn cleanup_cache(
    State(_state): State<AppState>,
    Json(request): Json<CacheCleanupRequest>,
) -> Result<Json<CacheCleanupResponse>, Json<CacheCleanupResponse>> {
    info!("Iniciando limpieza de cache: {}", request.cache_type);
    
    match request.cache_type.as_str() {
        "detail" => {
            // Crear cache de detalle
            let cache = crate::cache::DetailCache::new(crate::clients::DetailCacheConfig::default());
            
            match cache.cleanup_expired().await {
                Ok(cleaned) => {
                    info!("Cache de detalle limpiado: {} entradas", cleaned);
                    Ok(Json(CacheCleanupResponse {
                        success: true,
                        message: format!("Cache de detalle limpiado: {} entradas eliminadas", cleaned),
                        entries_cleaned: cleaned,
                    }))
                }
                Err(e) => {
                    warn!("Error limpiando cache de detalle: {}", e);
                    Ok(Json(CacheCleanupResponse {
                        success: false,
                        message: format!("Error limpiando cache: {}", e),
                        entries_cleaned: 0,
                    }))
                }
            }
        }
        "all" => {
            // Limpiar todos los caches
            let cache = crate::cache::DetailCache::new(crate::clients::DetailCacheConfig::default());
            
            match cache.clear().await {
                Ok(_) => {
                    info!("Todos los caches limpiados");
                    Ok(Json(CacheCleanupResponse {
                        success: true,
                        message: "Todos los caches limpiados exitosamente".to_string(),
                        entries_cleaned: 0, // No contamos las entradas en clear()
                    }))
                }
                Err(e) => {
                    warn!("Error limpiando todos los caches: {}", e);
                    Ok(Json(CacheCleanupResponse {
                        success: false,
                        message: format!("Error limpiando caches: {}", e),
                        entries_cleaned: 0,
                    }))
                }
            }
        }
        _ => {
            warn!("Tipo de cache no válido: {}", request.cache_type);
            Ok(Json(CacheCleanupResponse {
                success: false,
                message: "Tipo de cache no válido".to_string(),
                entries_cleaned: 0,
            }))
        }
    }
}

/// Obtener estadísticas del cache
pub async fn get_cache_stats(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, Json<serde_json::Value>> {
    info!("Obteniendo estadísticas del cache");
    
    let cache = crate::cache::DetailCache::new(crate::clients::DetailCacheConfig::default());
    
    match cache.get_stats().await {
        Ok(stats) => {
            let response = serde_json::json!({
                "success": true,
                "message": "Estadísticas obtenidas exitosamente",
                "data": {
                    "hits": stats.hits,
                    "misses": stats.misses,
                    "entries_created": stats.entries_created,
                    "entries_expired": stats.entries_expired,
                    "entries_evicted": stats.entries_evicted,
                    "hit_rate": if stats.hits + stats.misses > 0 {
                        stats.hits as f64 / (stats.hits + stats.misses) as f64
                    } else {
                        0.0
                    }
                }
            });
            
            Ok(Json(response))
        }
        Err(e) => {
            warn!("Error obteniendo estadísticas: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "message": format!("Error obteniendo estadísticas: {}", e),
                "data": null
            })))
        }
    }
}

/// Crear paquetes mock para testing
fn create_mock_packages(ref_colis_list: &[String]) -> Vec<crate::models::package::Package> {
    ref_colis_list
        .iter()
        .enumerate()
        .map(|(i, ref_colis)| crate::models::package::Package {
            id: uuid::Uuid::new_v4(),
            company_id: uuid::Uuid::new_v4(),
            tournee_id: uuid::Uuid::new_v4(),
            tracking_number: ref_colis.clone(),
            external_tracking_number: Some(format!("S{}", ref_colis)),
            package_origin: Some("colis_prive".to_string()),
            external_package_id: Some(ref_colis.clone()),
            integration_id: None,
            package_type: Some("colis".to_string()),
            package_weight: Some(rust_decimal::Decimal::new(100, 2)), // 1.00 kg
            package_dimensions: Some("10x10x10".to_string()),
            delivery_status: crate::models::package::DeliveryStatus::Pending,
            delivery_date: None,
            delivery_time: None,
            delivery_attempts: 0,
            recipient_name: Some(format!("Destinataire {}", i + 1)),
            recipient_phone: Some("0123456789".to_string()),
            delivery_address: format!("{} RUE TEST, 75018 PARIS, FRANCE", i + 1),
            delivery_instructions: Some("Entregar en horario comercial".to_string()),
            failure_reason: None,
            failure_notes: None,
            reschedule_date: None,
            delivery_photo: None,
            signature_required: true,
            signature_image: None,
            signature_photo: None,
            delivery_coordinates: Some(crate::models::package::Point {
                x: 2.3522 + (i as f64 * 0.001),
                y: 48.8566 + (i as f64 * 0.001),
            }),
            delivery_duration_minutes: None,
            driver_notes: None,
            package_condition: None,
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
            deleted_at: None,
        })
        .collect()
}
