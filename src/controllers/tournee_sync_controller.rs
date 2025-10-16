use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::cache::redis_client::RedisClient;
use chrono::{DateTime, Utc};

/// Estado compartido de la aplicación
pub type AppState = Arc<Mutex<RedisClient>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourneeState {
    pub tournee_id: String,
    pub version: u32,
    pub timestamp: DateTime<Utc>,
    pub problematic_packages: Vec<String>, // IDs de paquetes problemáticos
    pub updated_coords: Vec<PackageCoords>, // Coordenadas actualizadas por el chofer
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageCoords {
    pub package_id: String,
    pub lat: f64,
    pub lng: f64,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub tournee_id: String,
    pub version: u32,
    pub problematic_packages: Vec<String>,
    pub updated_coords: Vec<PackageCoords>,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub success: bool,
    pub server_version: u32,
    pub problematic_packages: Vec<String>,
    pub updated_coords: Vec<PackageCoords>,
    pub timestamp: DateTime<Utc>,
    pub message: Option<String>,
}

/// Sincronizar estado de la tournée (desde frontend)
pub async fn sync_tournee_state(
    State(state): State<AppState>,
    Path(tournee_id): Path<String>,
    Json(payload): Json<SyncRequest>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let redis = state.lock().await;
    
    // Clave de caché para el estado de la tournée
    let cache_key = format!("tournee_state:{}", tournee_id);
    
    // Obtener estado actual del servidor
    let current_state: Option<TourneeState> = redis.get(&cache_key).await
        .unwrap_or(None);
    
    match current_state {
        Some(mut server_state) => {
            // Verificar versión
            if payload.version < server_state.version {
                // Frontend está desactualizado
                log::warn!("⚠️ Frontend desactualizado: v{} < v{}", payload.version, server_state.version);
                
                return Ok(ResponseJson(json!({
                    "success": false,
                    "server_version": server_state.version,
                    "problematic_packages": server_state.problematic_packages,
                    "updated_coords": server_state.updated_coords,
                    "timestamp": server_state.timestamp,
                    "message": "Frontend desactualizado, usando estado del servidor"
                })));
            }
            
            // Mergear cambios del frontend
            for pkg_id in &payload.problematic_packages {
                if !server_state.problematic_packages.contains(pkg_id) {
                    server_state.problematic_packages.push(pkg_id.clone());
                    log::info!("➕ Paquete {} marcado como problemático", pkg_id);
                }
            }
            
            // Mergear coordenadas actualizadas
            for coords in &payload.updated_coords {
                // Remover coordenadas anteriores del mismo paquete
                server_state.updated_coords.retain(|c| c.package_id != coords.package_id);
                server_state.updated_coords.push(coords.clone());
                log::info!("📍 Coordenadas actualizadas para {}", coords.package_id);
            }
            
            // Incrementar versión
            server_state.version += 1;
            server_state.timestamp = Utc::now();
            server_state.checksum = payload.checksum;
            
            // Guardar en Redis (TTL de 24 horas)
            if let Err(e) = redis.set(&cache_key, &server_state, 86400).await {
                log::error!("❌ Error guardando estado en Redis: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            
            log::info!("✅ Estado sincronizado: tournée {} v{}", tournee_id, server_state.version);
            
            Ok(ResponseJson(json!({
                "success": true,
                "server_version": server_state.version,
                "problematic_packages": server_state.problematic_packages,
                "updated_coords": server_state.updated_coords,
                "timestamp": server_state.timestamp,
                "message": "Estado sincronizado exitosamente"
            })))
        }
        None => {
            // Crear nuevo estado
            let new_state = TourneeState {
                tournee_id: tournee_id.clone(),
                version: 1,
                timestamp: Utc::now(),
                problematic_packages: payload.problematic_packages.clone(),
                updated_coords: payload.updated_coords.clone(),
                checksum: payload.checksum.clone(),
            };
            
            // Guardar en Redis
            if let Err(e) = redis.set(&cache_key, &new_state, 86400).await {
                log::error!("❌ Error guardando estado en Redis: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            
            log::info!("✅ Nuevo estado creado: tournée {} v{}", tournee_id, new_state.version);
            
            Ok(ResponseJson(json!({
                "success": true,
                "server_version": new_state.version,
                "problematic_packages": new_state.problematic_packages,
                "updated_coords": new_state.updated_coords,
                "timestamp": new_state.timestamp,
                "message": "Nuevo estado creado"
            })))
        }
    }
}

/// Obtener estado de la tournée (desde servidor)
pub async fn get_tournee_state(
    State(state): State<AppState>,
    Path(tournee_id): Path<String>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let redis = state.lock().await;
    
    let cache_key = format!("tournee_state:{}", tournee_id);
    
    match redis.get::<TourneeState>(&cache_key).await {
        Ok(Some(server_state)) => {
            log::info!("✅ Estado encontrado: tournée {} v{}", tournee_id, server_state.version);
            
            Ok(ResponseJson(json!({
                "success": true,
                "server_version": server_state.version,
                "problematic_packages": server_state.problematic_packages,
                "updated_coords": server_state.updated_coords,
                "timestamp": server_state.timestamp,
                "message": "Estado obtenido exitosamente"
            })))
        }
        Ok(None) => {
            log::info!("ℹ️ No hay estado guardado para tournée {}", tournee_id);
            
            Ok(ResponseJson(json!({
                "success": true,
                "server_version": 0,
                "problematic_packages": [],
                "updated_coords": [],
                "timestamp": Utc::now(),
                "message": "No hay estado guardado"
            })))
        }
        Err(e) => {
            log::error!("❌ Error obteniendo estado: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Limpiar estado de tournée
pub async fn clear_tournee_state(
    State(state): State<AppState>,
    Path(tournee_id): Path<String>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let redis = state.lock().await;
    
    let cache_key = format!("tournee_state:{}", tournee_id);
    
    match redis.delete(&cache_key).await {
        Ok(_) => {
            log::info!("🗑️ Estado eliminado: tournée {}", tournee_id);
            
            Ok(ResponseJson(json!({
                "success": true,
                "message": "Estado eliminado exitosamente"
            })))
        }
        Err(e) => {
            log::error!("❌ Error eliminando estado: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

