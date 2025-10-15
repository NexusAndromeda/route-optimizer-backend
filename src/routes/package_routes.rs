use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
    routing::{get, put},
    Router,
};
use std::sync::Arc;
use serde::Deserialize;
use crate::services::package_processing_service::PackageProcessingService;
use crate::services::address_matching_service::AddressMatchingService;
use crate::services::colis_prive_service::ColisPriveService;
use crate::models::package::GroupedPackages;
use crate::state::AppState;
use tracing::{info, error};
use uuid::Uuid;

/// Obtiene paquetes agrupados para una empresa
pub async fn get_grouped_packages(
    State(app_state): State<AppState>,
) -> Result<Json<GroupedPackages>, (StatusCode, Json<serde_json::Value>)> {
    info!("📦 Solicitud de paquetes agrupados recibida");
    
    // Por ahora, usar un company_id fijo para testing
    let company_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
        .map_err(|_| {
            error!("❌ Error parseando company_id");
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid company ID"
            })))
        })?;
    
    // Obtener paquetes de Colis Privé
    let colis_service = ColisPriveService::new(app_state.http_client.clone(), app_state.config.clone());
    
    // Por ahora, retornamos una lista vacía de paquetes
    // Se implementará cuando tengamos la integración real
    let colis_packages: Vec<crate::models::package::ColisPrivePackage> = vec![];
    
    if colis_packages.is_empty() {
        info!("📭 No hay paquetes disponibles para la empresa (mock)");
        return Ok(Json(GroupedPackages::new()));
    }
    
    info!("📦 {} paquetes obtenidos de Colis Privé", colis_packages.len());
    
    // Crear servicios de procesamiento
    let address_matcher = match AddressMatchingService::new(Arc::new(app_state.pool.clone())).await {
        Ok(matcher) => matcher,
        Err(e) => {
            error!("❌ Error inicializando AddressMatchingService: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Error inicializando servicio de direcciones",
                "details": e.to_string()
            }))));
        }
    };
    
    let package_processor = PackageProcessingService::new(address_matcher);
    
    // Procesar y agrupar paquetes
    let grouped_packages = match package_processor.process_tournee(colis_packages, Some(company_id)).await {
        Ok(grouped) => grouped,
        Err(e) => {
            error!("❌ Error procesando paquetes: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Error procesando paquetes",
                "details": e.to_string()
            }))));
        }
    };
    
    info!("✅ Paquetes procesados: {} singles, {} groups, {} totales", 
        grouped_packages.singles.len(), 
        grouped_packages.groups.len(), 
        grouped_packages.total_packages);
    
    Ok(Json(grouped_packages))
}

/// Obtiene estadísticas de procesamiento
pub async fn get_processing_stats(
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    info!("📊 Solicitud de estadísticas de procesamiento");
    
    let address_matcher = match AddressMatchingService::new(Arc::new(app_state.pool.clone())).await {
        Ok(matcher) => matcher,
        Err(e) => {
            error!("❌ Error inicializando AddressMatchingService: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Error inicializando servicio de direcciones",
                "details": e.to_string()
            }))));
        }
    };
    
    let package_processor = PackageProcessingService::new(address_matcher);
    
    match package_processor.get_processing_stats().await {
        Ok((cache_size, sample_keys)) => {
            Ok(Json(serde_json::json!({
                "address_cache_size": cache_size,
                "sample_addresses": sample_keys,
                "timestamp": chrono::Utc::now()
            })))
        }
        Err(e) => {
            error!("❌ Error obteniendo estadísticas: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Error obteniendo estadísticas",
                "details": e.to_string()
            }))))
        }
    }
}

/// Actualiza datos del chofer para una dirección
pub async fn update_address_driver_data(
    Path(address_id): Path<Uuid>,
    State(app_state): State<AppState>,
    Json(update_data): Json<UpdateDriverDataRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    info!("🔄 Actualizando datos del chofer para dirección: {}", address_id);
    
    let address_matcher = match AddressMatchingService::new(Arc::new(app_state.pool.clone())).await {
        Ok(matcher) => matcher,
        Err(e) => {
            error!("❌ Error inicializando AddressMatchingService: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Error inicializando servicio de direcciones",
                "details": e.to_string()
            }))));
        }
    };
    
    match address_matcher.update_driver_data(
        address_id,
        update_data.door_code,
        update_data.has_mailbox_access,
        update_data.driver_notes,
        update_data.updated_by,
    ).await {
        Ok(updated_address) => {
            info!("✅ Datos del chofer actualizados para: {}", updated_address.official_label);
            Ok(Json(serde_json::to_value(updated_address).unwrap()))
        }
        Err(e) => {
            error!("❌ Error actualizando datos del chofer: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Error actualizando datos del chofer",
                "details": e.to_string()
            }))))
        }
    }
}

/// Configura las rutas de paquetes
pub fn package_routes() -> Router<AppState> {
    Router::new()
        .route("/packages/grouped", get(get_grouped_packages))
        .route("/packages/stats", get(get_processing_stats))
        .route("/addresses/:address_id/driver-data", put(update_address_driver_data))
}

#[derive(Deserialize)]
pub struct UpdateDriverDataRequest {
    pub door_code: Option<String>,
    pub has_mailbox_access: bool,
    pub driver_notes: Option<String>,
    pub updated_by: String,
}