use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
    http::StatusCode,
};
use std::sync::Arc;
use crate::controllers::colis_prive_controller::ColisPriveController;
use crate::dto::colis_prive_dto::*;
use crate::state::AppState;
use crate::utils::errors::AppError;
use crate::services::address_matching_service::AddressMatchingService;
use crate::services::package_processing_service::PackageProcessingService;
use crate::models::package::GroupedPackages;
use tracing::{info, error};

pub fn create_colis_prive_routes() -> Router<AppState> {
    Router::new()
        .route("/auth", post(authenticate))
        .route("/packages", post(get_packages))
        .route("/optimize", post(optimize_route))
        .route("/companies", get(get_companies))
        .route("/health", get(health_check))
}

async fn authenticate(
    State(state): State<AppState>,
    Json(request): Json<ColisPriveAuthRequest>,
) -> Json<ColisPriveAuthResponse> {
    let controller = ColisPriveController::new(&state);
    match controller.authenticate(request).await {
        Ok(response) => Json(response),
        Err(e) => Json(ColisPriveAuthResponse {
            success: false,
            message: None,
            authentication: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn get_packages(
    State(state): State<AppState>,
    Json(request): Json<GetPackagesRequest>,
) -> Result<Json<GroupedPackages>, AppError> {
    info!("📦 Solicitud de paquetes agrupados para: {}:{}", request.societe, request.matricule);
    
    // 1. Obtener paquetes de Colis Privé usando el controller existente
    let controller = ColisPriveController::new(&state);
    let packages_response = controller.get_packages(request, &state).await?;
    
    if packages_response.packages.is_empty() {
        info!("📭 No hay paquetes disponibles");
        return Ok(Json(GroupedPackages::new()));
    }
    
    info!("📦 {} paquetes obtenidos de Colis Privé", packages_response.packages.len());
    
    // 2. Inicializar servicio de matching de direcciones
    let address_matcher = AddressMatchingService::new(Arc::new(state.pool.clone()))
        .await
        .map_err(|e| {
            error!("❌ Error inicializando AddressMatchingService: {}", e);
            AppError::Internal(format!("Error inicializando servicio de direcciones: {}", e))
        })?;
    
    let package_processor = PackageProcessingService::new(address_matcher);
    
    // 3. Convertir PackageData a ColisPrivePackage
    let colis_packages: Vec<crate::models::package::ColisPrivePackage> = packages_response.packages
        .into_iter()
        .filter_map(|pkg| {
            let latitude = pkg.coord_y_destinataire.or(pkg.latitude).unwrap_or(48.8566);
            let longitude = pkg.coord_x_destinataire.or(pkg.longitude).unwrap_or(2.3522);
            let libelle_voie = pkg.destinataire_adresse1.clone().unwrap_or_default();
            let code_postal = pkg.destinataire_cp.clone().unwrap_or_default();
            
            if libelle_voie.is_empty() || code_postal.is_empty() {
                return None;
            }
            
            Some(crate::models::package::ColisPrivePackage {
                code_barre_article: pkg.reference_colis.clone(),
                destinataire_nom: pkg.destinataire_nom.clone(),
                destinataire_telephone: pkg.phone.or(pkg.phone_fixed),
                destinataire_indication: pkg.instructions.clone(),
                num_voie_geocode_livraison: None,
                libelle_voie_geocode_livraison: libelle_voie,
                code_postal_geocode_livraison: code_postal,
                latitude,
                longitude,
            })
        })
        .collect();
    
    info!("📦 {} paquetes válidos para procesar", colis_packages.len());
    
    // 4. Procesar y agrupar
    let grouped_packages = package_processor.process_tournee(colis_packages, None)
        .await
        .map_err(|e| {
            error!("❌ Error procesando paquetes: {}", e);
            AppError::Internal(format!("Error procesando paquetes: {}", e))
        })?;
    
    info!("✅ Paquetes procesados: {} singles, {} groups, {} totales", 
        grouped_packages.singles.len(), 
        grouped_packages.groups.len(), 
        grouped_packages.total_packages);
    
    Ok(Json(grouped_packages))
}

async fn optimize_route(
    State(state): State<AppState>,
    Json(request): Json<OptimizeRouteRequest>,
) -> Result<Json<OptimizeRouteResponse>, AppError> {
    let controller = ColisPriveController::new(&state);
    let response = controller.optimize_route(request, &state).await?;
    Ok(Json(response))
}

async fn get_companies() -> Result<Json<CompaniesListResponse>, AppError> {
    let response = ColisPriveController::get_companies().await?;
    Ok(Json(response))
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "colis-prive",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
