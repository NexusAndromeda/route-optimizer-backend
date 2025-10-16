use axum::{
    routing::{get, post, delete},
    Router,
};
use crate::controllers::tournee_sync_controller::{
    AppState, sync_tournee_state, get_tournee_state, clear_tournee_state
};

/// Configura las rutas de sincronización de tournées
pub fn tournee_sync_routes() -> Router<AppState> {
    Router::new()
        .route("/state/:tournee_id", get(get_tournee_state))
        .route("/state/:tournee_id", post(sync_tournee_state))
        .route("/state/:tournee_id", delete(clear_tournee_state))
}

