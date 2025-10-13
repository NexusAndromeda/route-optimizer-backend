use axum::Router;
use crate::state::AppState;

pub fn create_vehicle_router() -> Router<AppState> {
    Router::new()
    // TODO: Add vehicle routes
}

