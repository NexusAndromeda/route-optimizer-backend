use axum::Router;
use crate::state::AppState;
use crate::api::colis_prive_router::create_colis_prive_router;

pub fn create_colis_prive_routes() -> Router<AppState> {
    // Por ahora, usar el router existente
    create_colis_prive_router()
}

