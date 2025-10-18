use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
    routing::{get, put, post},
    Router,
};
use std::sync::Arc;
use serde::Deserialize;
use crate::services::package_processing_service::PackageProcessingService;
use crate::services::address_matching_service::AddressMatchingService;
use crate::controllers::colis_prive_controller::ColisPriveController;
use crate::dto::colis_prive_dto::GetPackagesRequest;
use crate::models::package::GroupedPackages;
use crate::state::AppState;
use crate::utils::errors::AppError;
use tracing::{info, error};
use uuid::Uuid;

/// Obtiene paquetes agrupados de Colis Privé
pub async fn get_grouped_packages(
    State(app_state): State<AppState>,
    Json(request): Json<GetPackagesRequest>,
) -> Result<Json<GroupedPackages>, (StatusCode, Json<serde_json::Value>)> {
    info!("📦 Solicitud de paquetes agrupados recibida para: {}:{}", 
        request.societe, request.matricule);
    
    // 1. Obtener paquetes de Colis Privé usando el controller existente
    let controller = ColisPriveController::new(&app_state);
    let packages_response = match controller.get_packages(request, &app_state).await {
        Ok(response) => response,
        Err(e) => {
            error!("❌ Error obteniendo paquetes de Colis Privé: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Error obteniendo paquetes de Colis Privé",
                "details": e.to_string()
            }))));
        }
    };
    
    // 2. Convertir paquetes de Colis Privé al formato que necesitamos
    // Por ahora, si no hay paquetes, retornar vacío
    if packages_response.packages.is_empty() {
        info!("📭 No hay paquetes disponibles");
        return Ok(Json(GroupedPackages::new()));
    }
    
    info!("📦 {} paquetes obtenidos de Colis Privé", packages_response.packages.len());
    
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
    
    // 3. Convertir PackageData de Colis Privé a ColisPrivePackage
    let colis_packages: Vec<crate::models::package::ColisPrivePackage> = packages_response.packages
        .into_iter()
        .filter_map(|pkg| {
            // Extraer coordenadas
            let latitude = pkg.coord_y_destinataire.or(pkg.latitude).unwrap_or(48.8566); // Default París
            let longitude = pkg.coord_x_destinataire.or(pkg.longitude).unwrap_or(2.3522);
            
            // Extraer dirección
            let libelle_voie = pkg.destinataire_adresse1.clone().unwrap_or_default();
            let code_postal = pkg.destinataire_cp.clone().unwrap_or_default();
            
            // Si no tiene coordenadas válidas o dirección, lo ignoramos
            if libelle_voie.is_empty() || code_postal.is_empty() {
                return None;
            }
            
            Some(crate::models::package::ColisPrivePackage {
                code_barre_article: pkg.reference_colis.clone(),
                destinataire_nom: pkg.destinataire_nom.clone(),
                destinataire_telephone: pkg.phone.or(pkg.phone_fixed),
                destinataire_indication: pkg.instructions.clone(),
                
                // GeocodeDestinataire (no disponibles en mock)
                num_voie_geocode_destinataire: None,
                libelle_voie_geocode_destinataire: None,
                code_postal_geocode_destinataire: None,
                qualite_geocodage_destinataire: Some("Bon".to_string()), // Asumir buena calidad para mock
                
                // OrigineDestinataire (fallback)
                libelle_voie_origine_destinataire: Some(libelle_voie.clone()),
                code_postal_origine_destinataire: Some(code_postal.clone()),
                
                // GeocodeLivraison (legacy - usar como fallback)
                num_voie_geocode_livraison: None,
                libelle_voie_geocode_livraison: Some(libelle_voie),
                code_postal_geocode_livraison: Some(code_postal),
                
                latitude,
                longitude,
                code_statut_article: None, // No disponible en formato mock
            })
        })
        .collect();
    
    info!("📦 {} paquetes válidos para procesar", colis_packages.len());
    
    // Procesar y agrupar paquetes
    let grouped_packages = match package_processor.process_tournee(colis_packages, None).await {
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
        .route("/packages/grouped", post(get_grouped_packages))
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