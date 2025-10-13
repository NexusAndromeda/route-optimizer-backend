use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use crate::controllers::vehicle_controller::VehicleController;
use crate::dto::vehicle_dto::{CreateVehicleRequest, UpdateVehicleRequest, VehicleResponse};
use crate::dto::company_dto::ApiResponse;
use crate::state::AppState;
use crate::utils::errors::AppError;
use uuid::Uuid;

pub fn create_vehicle_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_vehicle))
        .route("/", get(list_vehicles))
        .route("/:id", get(get_vehicle))
        .route("/:id", put(update_vehicle))
        .route("/:id", delete(delete_vehicle))
}

// TODO: Extraer company_id del JWT token cuando implementemos middleware de auth
// Por ahora usamos un company_id hardcoded de ejemplo
async fn get_company_id_from_jwt() -> Uuid {
    // Placeholder - en producción esto vendría del JWT
    Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
}

async fn create_vehicle(
    State(state): State<AppState>,
    Json(request): Json<CreateVehicleRequest>,
) -> Result<Json<ApiResponse<VehicleResponse>>, AppError> {
    let company_id = get_company_id_from_jwt().await; // TODO: Extraer del JWT
    let controller = VehicleController::new(state.pool.clone());
    let response = controller.create(company_id, request).await?;
    Ok(Json(response))
}

async fn get_vehicle(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleResponse>, AppError> {
    let company_id = get_company_id_from_jwt().await; // TODO: Extraer del JWT
    let controller = VehicleController::new(state.pool.clone());
    let response = controller.get_by_id(id, company_id).await?;
    Ok(Json(response))
}

async fn list_vehicles(
    State(state): State<AppState>,
) -> Result<Json<Vec<VehicleResponse>>, AppError> {
    let company_id = get_company_id_from_jwt().await; // TODO: Extraer del JWT
    let controller = VehicleController::new(state.pool.clone());
    let response = controller.list_by_company(company_id).await?;
    Ok(Json(response))
}

async fn update_vehicle(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateVehicleRequest>,
) -> Result<Json<ApiResponse<VehicleResponse>>, AppError> {
    let company_id = get_company_id_from_jwt().await; // TODO: Extraer del JWT
    let controller = VehicleController::new(state.pool.clone());
    let response = controller.update(id, company_id, request).await?;
    Ok(Json(response))
}

async fn delete_vehicle(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let company_id = get_company_id_from_jwt().await; // TODO: Extraer del JWT
    let controller = VehicleController::new(state.pool.clone());
    controller.delete(id, company_id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Vehículo eliminado exitosamente"
    })))
}
