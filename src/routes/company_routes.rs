use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use crate::controllers::company_controller::CompanyController;
use crate::dto::company_dto::{RegisterCompanyRequest, CompanyResponse, ApiResponse};
use crate::dto::auth_dto::{LoginRequest, LoginResponse};
use crate::state::AppState;
use crate::utils::errors::AppError;

pub fn create_company_router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/me", get(get_current_company))
}

async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterCompanyRequest>,
) -> Result<Json<ApiResponse<CompanyResponse>>, AppError> {
    let controller = CompanyController::new(state.pool.clone());
    let response = controller.register(request).await?;
    Ok(Json(response))
}

async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let controller = CompanyController::new(state.pool.clone());
    let response = controller.login(request).await?;
    Ok(Json(response))
}

async fn get_current_company(
    State(_state): State<AppState>,
    // TODO: Extraer company_id del JWT
) -> Result<Json<CompanyResponse>, AppError> {
    // Implementar cuando tengamos middleware de auth
    Err(AppError::NotImplemented("Not implemented yet".to_string()))
}

