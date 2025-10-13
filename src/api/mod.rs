//! API endpoints legacy
//! 
//! Este módulo contiene endpoints legacy que aún se mantienen.
//! Los nuevos endpoints están en src/routes/

pub mod geocoding;

use axum::Router;
use crate::state::AppState;

/// Crear el router legacy de la API
pub fn create_legacy_api_router() -> Router<AppState> {
    Router::new()
        .nest("/api/geocoding", geocoding::create_geocoding_router())
}