use axum::{
    routing::{post, get},
    Router,
};
use crate::controllers::auth_controller::{AppState, login, refresh_token, logout, validate_token, get_active_sessions, validate_auth_header};

/// Configura las rutas de autenticación
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        .route("/validate", post(validate_token))
        .route("/sessions", get(get_active_sessions))
        .route("/me", get(validate_auth_header))
}
