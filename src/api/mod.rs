//! API endpoints
//! 
//! Este módulo contiene los endpoints de la API.

pub mod colis_prive;
pub mod colis_prive_router;
pub mod geocoding;
pub mod hybrid;
pub mod auth;

pub use colis_prive_router::*;

use axum::Router;
use crate::state::AppState;

/// Crear el router principal de la API
pub fn create_api_router() -> Router<AppState> {
    Router::new()
        .nest("/colis-prive", create_colis_prive_router())
        .nest("/api", geocoding::create_geocoding_router())
        .merge(hybrid::create_router())
        .merge(auth::create_auth_router())
}