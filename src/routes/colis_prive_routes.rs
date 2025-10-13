use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use crate::controllers::colis_prive_controller::ColisPriveController;
use crate::dto::colis_prive_dto::*;
use crate::state::AppState;
use crate::utils::errors::AppError;

pub fn create_colis_prive_routes() -> Router<AppState> {
    Router::new()
        .route("/auth", post(authenticate))
        .route("/packages", post(get_packages))
        .route("/optimize", post(optimize_route))
        .route("/companies", get(get_companies))
        .route("/health", get(health_check))
}

async fn authenticate(
    State(state): State<AppState>,
    Json(request): Json<ColisPriveAuthRequest>,
) -> Result<Json<ColisPriveAuthResponse>, AppError> {
    let controller = ColisPriveController::new(&state);
    let response = controller.authenticate(request).await?;
    Ok(Json(response))
}

async fn get_packages(
    State(state): State<AppState>,
    Json(request): Json<GetPackagesRequest>,
) -> Result<Json<PackagesResponse>, AppError> {
    let controller = ColisPriveController::new(&state);
    let response = controller.get_packages(request, &state).await?;
    Ok(Json(response))
}

async fn optimize_route(
    State(state): State<AppState>,
    Json(request): Json<OptimizeRouteRequest>,
) -> Result<Json<OptimizeRouteResponse>, AppError> {
    let controller = ColisPriveController::new(&state);
    let response = controller.optimize_route(request, &state).await?;
    Ok(Json(response))
}

async fn get_companies() -> Result<Json<CompaniesListResponse>, AppError> {
    let response = ColisPriveController::get_companies().await?;
    Ok(Json(response))
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "colis-prive",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
