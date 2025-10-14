//! Rutas para Mapbox Optimization API
//! 
//! Este módulo define las rutas relacionadas con la optimización de rutas
//! usando la API de Mapbox Optimization.

use axum::{
    routing::{get, post},
    Router,
};

use crate::controllers::mapbox_optimization_controller;
use crate::state::AppState;

/// Crear el router para las rutas de Mapbox Optimization
pub fn create_mapbox_optimization_routes() -> Router<AppState> {
    Router::new()
        .route("/optimize", post(mapbox_optimization_controller::optimize_route))
        .route("/health", get(mapbox_optimization_controller::health_check))
        .route("/info", get(mapbox_optimization_controller::service_info))
}
