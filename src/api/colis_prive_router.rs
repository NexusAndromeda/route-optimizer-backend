use axum::{
    Router,
    routing::{post, get},
};
use crate::api::colis_prive::*;
use crate::state::AppState;

/// Crear el router para endpoints de Colis Privé (API Web)
pub fn create_colis_prive_router() -> Router<AppState> {
    Router::new()
        .route("/auth", post(authenticate_colis_prive))     // Autenticación
        .route("/tournee", post(get_tournee_data))          // Tournée (API Web)
        .route("/optimize", post(optimize_tournee))         // Optimización de ruta
        .route("/health", get(health_check_colis_prive))    // Health check
}
