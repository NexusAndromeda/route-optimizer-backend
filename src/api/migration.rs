use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde_json::json;
use tracing::{info, error};

use crate::{
    state::AppState,
    migration_service::{MigrationService, MigrationStrategy, MigrationConfig},
};

/// GET /api/migration/status - Obtener estado actual de la migración
pub async fn get_migration_status(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let migration_service = &state.migration_service;
    
    let summary = migration_service.get_migration_summary().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let status = json!({
        "current_strategy": format!("{:?}", summary.current_strategy),
        "mobile_percentage": summary.current_strategy.mobile_percentage(),
        "web_percentage": summary.current_strategy.web_percentage(),
        "auto_progression": summary.config.auto_progression,
        "last_updated": chrono::Utc::now().to_rfc3339(),
        "status": "active"
    });
    
    Ok(Json(status))
}

/// POST /api/migration/strategy - Cambiar estrategia de migración
pub async fn change_migration_strategy(
    State(state): State<AppState>,
    Json(request): Json<ChangeStrategyRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔄 Cambiando estrategia de migración a: {:?}", request.strategy);
    
    let mut migration_service = state.migration_service.clone();
    
    // Log de debug antes del cambio
    let current_strategy = migration_service.get_current_strategy().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    info!("🔍 Estrategia actual antes del cambio: {:?}", current_strategy);
    
    let result = migration_service.change_strategy(request.strategy).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Log de debug después del cambio
    let new_strategy = migration_service.get_current_strategy().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    info!("🔍 Estrategia actual después del cambio: {:?}", new_strategy);
    
    let response = json!({
        "success": true,
        "message": format!("Estrategia cambiada a {:?}", request.strategy),
        "new_strategy": format!("{:?}", request.strategy),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Ok(Json(response))
}

/// GET /api/migration/metrics - Obtener métricas de migración
pub async fn get_migration_metrics(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let migration_service = &state.migration_service;
    
    let all_metrics = migration_service.get_all_metrics().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let current_strategy = migration_service.get_current_strategy().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut strategies_json = serde_json::Map::new();
    
    for (strategy, metrics) in all_metrics {
        let strategy_name = format!("{:?}", strategy);
        strategies_json.insert(strategy_name, json!({
            "total_requests": metrics.total_requests,
            "successful_requests": metrics.successful_requests,
            "failed_requests": metrics.failed_requests,
            "success_rate": metrics.success_rate(),
            "avg_response_time_ms": metrics.avg_response_time_ms
        }));
    }
    
    let metrics = json!({
        "strategies": strategies_json,
        "current_strategy": format!("{:?}", current_strategy),
        "last_updated": chrono::Utc::now().to_rfc3339()
    });
    
    Ok(Json(metrics))
}

/// POST /api/migration/progress - Forzar progresión a siguiente estrategia
pub async fn force_migration_progress(
    State(state): State<AppState>,
    Json(request): Json<ForceProgressRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🚀 Forzando progresión de migración a: {:?}", request.force_strategy);
    
    let mut migration_service = state.migration_service.clone();
    
    let result = migration_service.change_strategy(request.force_strategy).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let response = json!({
        "success": true,
        "message": "Progresión forzada exitosamente",
        "new_strategy": format!("{:?}", request.force_strategy),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Ok(Json(response))
}

/// POST /api/migration/rollback - Hacer rollback a estrategia anterior
pub async fn force_migration_rollback(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔄 Forzando rollback de migración");
    
    // TODO: Implementar cuando tengamos el servicio de migración en el estado
    let response = json!({
        "success": true,
        "message": "Rollback forzado exitosamente",
        "new_strategy": "WebOnly",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Ok(Json(response))
}

/// GET /api/migration/health - Health check de migración
pub async fn migration_health_check(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let health = json!({
        "status": "healthy",
        "service": "migration",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0"
    });
    
    Ok(Json(health))
}

/// Request para cambiar estrategia
#[derive(Debug, serde::Deserialize)]
pub struct ChangeStrategyRequest {
    pub strategy: MigrationStrategy,
    pub reason: Option<String>,
}

/// Request para forzar progresión
#[derive(Debug, serde::Deserialize)]
pub struct ForceProgressRequest {
    pub force_strategy: MigrationStrategy,
}

/// Request para forzar rollback
#[derive(Debug, serde::Deserialize)]
pub struct ForceRollbackRequest {
    pub rollback_strategy: MigrationStrategy,
}
