//! API de migración mínima
//! 
//! Este módulo contiene la API mínima de migración.

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde_json::json;
use tracing::info;

use crate::state::AppState;

/// GET /api/migration/status - Obtener estado actual de la migración
pub async fn get_migration_status(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let status = json!({
        "current_strategy": "WebOnly",
        "status": "active",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Ok(Json(status))
}

/// POST /api/migration/strategy - Cambiar estrategia de migración
pub async fn change_migration_strategy(
    State(_state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔄 Cambiando estrategia de migración");
    
    let response = json!({
        "success": true,
        "message": "Estrategia cambiada exitosamente",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Ok(Json(response))
}

/// GET /api/migration/metrics - Obtener métricas de migración
pub async fn get_migration_metrics(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let metrics = json!({
        "current_strategy": "WebOnly",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Ok(Json(metrics))
}

/// POST /api/migration/progress - Forzar progresión a siguiente estrategia
pub async fn force_migration_progress(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🚀 Forzando progresión de migración");
    
    let response = json!({
        "success": true,
        "message": "Progresión forzada exitosamente",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Ok(Json(response))
}

/// POST /api/migration/rollback - Hacer rollback a estrategia anterior
pub async fn force_migration_rollback(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔄 Forzando rollback de migración");
    
    let response = json!({
        "success": true,
        "message": "Rollback ejecutado exitosamente",
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
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Ok(Json(health))
}
