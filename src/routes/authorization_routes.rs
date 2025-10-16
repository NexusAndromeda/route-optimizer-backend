use axum::{
    routing::{get, post},
    Router,
};
use crate::controllers::authorization_controller::{
    AppState, get_user_permissions, get_available_roles, check_permission,
    get_access_level, check_company_access, check_tournee_access, get_permission_stats
};

/// Configura las rutas de autorización
pub fn authorization_routes() -> Router<AppState> {
    Router::new()
        .route("/roles", get(get_available_roles))
        .route("/permissions/:user_id", get(get_user_permissions))
        .route("/check", post(check_permission))
        .route("/access-level", post(get_access_level))
        .route("/company-access", post(check_company_access))
        .route("/tournee-access", post(check_tournee_access))
        .route("/stats", get(get_permission_stats))
}
