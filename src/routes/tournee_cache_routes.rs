use axum::{
    routing::{get, post, put},
    Router,
};
use crate::controllers::tournee_cache_controller::{
    AppState, get_tournee, sync_tournee, get_company_tournees, 
    get_cache_stats, cleanup_cache, update_cache_config, get_sync_status
};

/// Configura las rutas de caché de tournées
pub fn tournee_cache_routes() -> Router<AppState> {
    Router::new()
        .route("/tournee/:tournee_id", get(get_tournee))
        .route("/sync", post(sync_tournee))
        .route("/company/:company_id/tournees", get(get_company_tournees))
        .route("/stats", get(get_cache_stats))
        .route("/cleanup", post(cleanup_cache))
        .route("/config", put(update_cache_config))
        .route("/sync-status/:tournee_id", get(get_sync_status))
}
