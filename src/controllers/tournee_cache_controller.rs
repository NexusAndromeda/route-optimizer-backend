use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::Json as ResponseJson,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::models::auth::TourneeSyncRequest;
use crate::services::tournee_cache_service::TourneeCacheStats;
use crate::services::tournee_cache_service::{TourneeCacheService, TourneeCacheConfig, ConflictResolutionStrategy};
use crate::services::auth_service::AuthService;
use crate::services::authorization_service::AuthorizationService;

/// Estado compartido de la aplicación
pub type AppState = Arc<Mutex<TourneeCacheService>>;

/// Obtiene una tournée del caché
pub async fn get_tournee(
    State(state): State<AppState>,
    Path(tournee_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let token = payload.get("token")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let mut cache_service = state.lock().await;
    
    // Validar token y obtener información del usuario
    let auth_service = AuthService::new().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_info = auth_service.validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Verificar permisos
    let authz_service = AuthorizationService::new(&auth_service);
    if !authz_service.can_access_tournee(&user_info, &tournee_id) {
        return Ok(ResponseJson(json!({
            "success": false,
            "message": "No tienes permisos para acceder a esta tournée"
        })));
    }

    match cache_service.get_tournee(&tournee_id).await {
        Ok(Some(tournee)) => {
            Ok(ResponseJson(json!({
                "success": true,
                "data": {
                    "tournee": tournee
                },
                "message": "Tournée obtenida exitosamente"
            })))
        }
        Ok(None) => {
            Ok(ResponseJson(json!({
                "success": false,
                "message": "Tournée no encontrada"
            })))
        }
        Err(e) => {
            log::error!("Error obteniendo tournée: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Sincroniza una tournée
pub async fn sync_tournee(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let token = payload.get("token")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let sync_request: TourneeSyncRequest = serde_json::from_value(
        payload.get("sync_request")
            .ok_or(StatusCode::BAD_REQUEST)?
            .clone()
    ).map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut cache_service = state.lock().await;
    
    // Validar token y obtener información del usuario
    let auth_service = AuthService::new().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_info = auth_service.validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    match cache_service.sync_tournee(&user_info, sync_request).await {
        Ok(response) => {
            Ok(ResponseJson(json!({
                "success": response.success,
                "data": {
                    "tournee": response.tournee,
                    "conflicts": response.conflicts
                },
                "message": response.message
            })))
        }
        Err(e) => {
            log::error!("Error sincronizando tournée: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Obtiene todas las tournées de una empresa
pub async fn get_company_tournees(
    State(state): State<AppState>,
    Path(company_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let token = payload.get("token")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let cache_service = state.lock().await;
    
    // Validar token y obtener información del usuario
    let auth_service = AuthService::new().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_info = auth_service.validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Verificar permisos
    let authz_service = AuthorizationService::new(&auth_service);
    if !authz_service.can_access_company(&user_info, &company_id) {
        return Ok(ResponseJson(json!({
            "success": false,
            "message": "No tienes permisos para acceder a esta empresa"
        })));
    }

    match cache_service.get_company_tournees(&company_id).await {
        Ok(response) => {
            Ok(ResponseJson(json!({
                "success": response.success,
                "data": {
                    "tournees": response.tournees
                },
                "message": response.message
            })))
        }
        Err(e) => {
            log::error!("Error obteniendo tournées de empresa: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Obtiene estadísticas del caché
pub async fn get_cache_stats(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let token = payload.get("token")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let cache_service = state.lock().await;
    
    // Validar token y obtener información del usuario
    let auth_service = AuthService::new().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_info = auth_service.validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Solo admins pueden ver estadísticas
    let authz_service = AuthorizationService::new(&auth_service);
    if !matches!(user_info.role, crate::models::auth::UserRole::SuperAdmin | crate::models::auth::UserRole::Admin) {
        return Ok(ResponseJson(json!({
            "success": false,
            "message": "No tienes permisos para ver estadísticas del sistema"
        })));
    }

    match cache_service.get_cache_stats().await {
        Ok(stats) => {
            Ok(ResponseJson(json!({
                "success": true,
                "data": {
                    "memory_entries": stats.memory_entries,
                    "redis_entries": stats.redis_entries,
                    "memory_hit_rate": stats.memory_hit_rate,
                    "redis_hit_rate": stats.redis_hit_rate,
                    "last_cleanup": stats.last_cleanup,
                    "config": {
                        "memory_cache_ttl_minutes": stats.config.memory_cache_ttl.num_minutes(),
                        "redis_cache_ttl_hours": stats.config.redis_cache_ttl.num_hours(),
                        "max_memory_entries": stats.config.max_memory_entries,
                        "auto_sync_interval_minutes": stats.config.auto_sync_interval.num_minutes(),
                        "conflict_resolution": format!("{:?}", stats.config.conflict_resolution)
                    }
                },
                "message": "Estadísticas obtenidas exitosamente"
            })))
        }
        Err(e) => {
            log::error!("Error obteniendo estadísticas: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Limpia el caché expirado
pub async fn cleanup_cache(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let token = payload.get("token")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let mut cache_service = state.lock().await;
    
    // Validar token y obtener información del usuario
    let auth_service = AuthService::new().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_info = auth_service.validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Solo admins pueden limpiar el caché
    let authz_service = AuthorizationService::new(&auth_service);
    if !matches!(user_info.role, crate::models::auth::UserRole::SuperAdmin | crate::models::auth::UserRole::Admin) {
        return Ok(ResponseJson(json!({
            "success": false,
            "message": "No tienes permisos para limpiar el caché"
        })));
    }

    match cache_service.cleanup_expired_cache().await {
        Ok(cleaned_count) => {
            Ok(ResponseJson(json!({
                "success": true,
                "data": {
                    "cleaned_entries": cleaned_count
                },
                "message": format!("Caché limpiado exitosamente: {} entradas removidas", cleaned_count)
            })))
        }
        Err(e) => {
            log::error!("Error limpiando caché: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Actualiza la configuración del caché
pub async fn update_cache_config(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let token = payload.get("token")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let mut cache_service = state.lock().await;
    
    // Validar token y obtener información del usuario
    let auth_service = AuthService::new().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_info = auth_service.validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Solo super admins pueden cambiar la configuración
    let authz_service = AuthorizationService::new(&auth_service);
    if !matches!(user_info.role, crate::models::auth::UserRole::SuperAdmin) {
        return Ok(ResponseJson(json!({
            "success": false,
            "message": "No tienes permisos para cambiar la configuración del sistema"
        })));
    }

    // Parsear nueva configuración
    let config = TourneeCacheConfig {
        memory_cache_ttl: chrono::Duration::minutes(
            payload.get("memory_cache_ttl_minutes")
                .and_then(|v| v.as_i64())
                .unwrap_or(30) as i64
        ),
        redis_cache_ttl: chrono::Duration::hours(
            payload.get("redis_cache_ttl_hours")
                .and_then(|v| v.as_i64())
                .unwrap_or(24) as i64
        ),
        max_memory_entries: payload.get("max_memory_entries")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000) as usize,
        auto_sync_interval: chrono::Duration::minutes(
            payload.get("auto_sync_interval_minutes")
                .and_then(|v| v.as_i64())
                .unwrap_or(5) as i64
        ),
        conflict_resolution: match payload.get("conflict_resolution")
            .and_then(|v| v.as_str())
            .unwrap_or("timestamp_wins") {
            "server_wins" => ConflictResolutionStrategy::ServerWins,
            "client_wins" => ConflictResolutionStrategy::ClientWins,
            "timestamp_wins" => ConflictResolutionStrategy::TimestampWins,
            "merge" => ConflictResolutionStrategy::Merge,
            _ => ConflictResolutionStrategy::TimestampWins,
        },
    };

    cache_service.update_config(config);

    Ok(ResponseJson(json!({
        "success": true,
        "message": "Configuración del caché actualizada exitosamente"
    })))
}

/// Obtiene el estado de sincronización de una tournée
pub async fn get_sync_status(
    State(state): State<AppState>,
    Path(tournee_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let token = payload.get("token")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let mut cache_service = state.lock().await;
    
    // Validar token y obtener información del usuario
    let auth_service = AuthService::new().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_info = auth_service.validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Verificar permisos
    let authz_service = AuthorizationService::new(&auth_service);
    if !authz_service.can_access_tournee(&user_info, &tournee_id) {
        return Ok(ResponseJson(json!({
            "success": false,
            "message": "No tienes permisos para acceder a esta tournée"
        })));
    }

    match cache_service.get_tournee(&tournee_id).await {
        Ok(Some(tournee)) => {
            let is_synced = tournee.last_activity > chrono::Utc::now() - chrono::Duration::minutes(5);
            Ok(ResponseJson(json!({
                "success": true,
                "data": {
                    "tournee_id": tournee_id,
                    "is_synced": is_synced,
                    "last_activity": tournee.last_activity,
                    "version": tournee.version,
                    "status": tournee.status
                },
                "message": "Estado de sincronización obtenido"
            })))
        }
        Ok(None) => {
            Ok(ResponseJson(json!({
                "success": false,
                "message": "Tournée no encontrada"
            })))
        }
        Err(e) => {
            log::error!("Error obteniendo estado de sincronización: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
