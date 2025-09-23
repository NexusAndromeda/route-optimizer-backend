use axum::Router;
use crate::state::AppState;

/// Crear el router de companies
pub fn create_companies_router() -> Router<AppState> {
    Router::new()
}

/// Crear el router de users
pub fn create_users_router() -> Router<AppState> {
    Router::new()
}

/// Crear el router de vehicles
pub fn create_vehicles_router() -> Router<AppState> {
    Router::new()
}

/// Crear el router de tournees
pub fn create_tournees_router() -> Router<AppState> {
    Router::new()
}

/// Crear el router de packages
pub fn create_packages_router() -> Router<AppState> {
    Router::new()
}

/// Crear el router de analytics
pub fn create_analytics_router() -> Router<AppState> {
    Router::new()
}
