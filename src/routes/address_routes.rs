use axum::Router;
use crate::state::AppState;

pub fn create_address_router() -> Router<AppState> {
    Router::new()
    // TODO: Add address routes
}

