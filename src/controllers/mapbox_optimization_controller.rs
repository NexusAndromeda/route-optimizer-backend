//! Controlador para optimización con Mapbox
//! 
//! Este módulo maneja los endpoints relacionados con la optimización de rutas
//! usando la API de Mapbox Optimization.

use axum::{
    extract::State,
    response::Json,
};
use serde_json::json;

use crate::dto::mapbox_optimization_dto::*;
use crate::services::mapbox_optimization_service::MapboxOptimizationService;
use crate::state::AppState;
use crate::utils::errors::AppError;

/// Optimizar ruta usando Mapbox Optimization API
pub async fn optimize_route(
    State(state): State<AppState>,
    Json(request): Json<OptimizationRequest>,
) -> Result<Json<OptimizationResponse>, AppError> {
    log::info!("🎯 Recibida solicitud de optimización Mapbox para {} paquetes", request.packages.len());

    // Verificar que tenemos el token de Mapbox
    let mapbox_token = match &state.config.mapbox_token {
        Some(token) => token.clone(),
        None => {
            log::error!("❌ MAPBOX_TOKEN no configurado");
            return Ok(Json(OptimizationResponse {
                success: false,
                message: Some("Mapbox token no configurado".to_string()),
                data: None,
            }));
        }
    };

    // Crear servicio de optimización
    let optimization_service = MapboxOptimizationService::new(mapbox_token);

    // Para este endpoint, usamos la ubicación por defecto de París como warehouse
    // En una implementación real, esto vendría de la configuración de la empresa
    let warehouse_location = Some((2.3522, 48.8566)); // París centro

    // Ejecutar optimización
    match optimization_service.optimize_route(request.packages, warehouse_location).await {
        Ok(response) => {
            log::info!("✅ Optimización Mapbox completada exitosamente");
            Ok(Json(response))
        }
        Err(e) => {
            log::error!("❌ Error en optimización Mapbox: {}", e);
            Err(AppError::ExternalApi(format!("Error en optimización Mapbox: {}", e)))
        }
    }
}

/// Health check para el servicio de optimización Mapbox
pub async fn health_check() -> Result<Json<serde_json::Value>, AppError> {
    log::info!("🏥 Health check Mapbox Optimization");
    
    Ok(Json(json!({
        "status": "ok",
        "service": "mapbox_optimization",
        "message": "Mapbox Optimization API service is running"
    })))
}

/// Obtener información sobre el servicio de optimización
pub async fn service_info() -> Result<Json<serde_json::Value>, AppError> {
    log::info!("ℹ️ Service info Mapbox Optimization");
    
    Ok(Json(json!({
        "service": "Mapbox Optimization API v2",
        "description": "Servicio de optimización de rutas usando Mapbox Optimization API",
        "features": [
            "Optimización de rutas multi-parada",
            "Hasta 1000 ubicaciones por request",
            "Ventanas de tiempo",
            "Capacidades de vehículos",
            "100,000 requests gratuitos por mes"
        ],
        "endpoints": [
            "POST /api/mapbox-optimization/optimize - Optimizar ruta",
            "GET /api/mapbox-optimization/health - Health check",
            "GET /api/mapbox-optimization/info - Información del servicio"
        ]
    })))
}
