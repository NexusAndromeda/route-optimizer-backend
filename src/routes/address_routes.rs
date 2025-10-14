use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use crate::controllers::address_controller::AddressController;
use crate::dto::address_dto::{SaveAddressRequest, AddressResponse, SearchAddressRequest};
use crate::dto::company_dto::ApiResponse;
use crate::state::AppState;
use crate::utils::errors::AppError;
use uuid::Uuid;
use serde::Deserialize;

pub fn create_address_router() -> Router<AppState> {
    Router::new()
        .route("/", post(save_address))
        .route("/search", get(search_addresses))
        .route("/geocode", post(geocode_address))
        .route("/:id", get(get_address))
        .route("/:id", put(update_address_details))
        .route("/:id", delete(delete_address))
        .route("/route/:route_id", get(list_by_route))
}

async fn save_address(
    State(state): State<AppState>,
    Json(request): Json<SaveAddressRequest>,
) -> Result<Json<ApiResponse<AddressResponse>>, AppError> {
    let controller = AddressController::new(state.pool.clone());
    let response = controller.save(request).await?;
    Ok(Json(response))
}

async fn get_address(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AddressResponse>, AppError> {
    let controller = AddressController::new(state.pool.clone());
    let response = controller.get_by_id(id).await?;
    Ok(Json(response))
}

async fn list_by_route(
    State(state): State<AppState>,
    Path(route_id): Path<Uuid>,
) -> Result<Json<Vec<AddressResponse>>, AppError> {
    let controller = AddressController::new(state.pool.clone());
    let response = controller.list_by_route(route_id).await?;
    Ok(Json(response))
}

async fn search_addresses(
    State(state): State<AppState>,
    Query(request): Query<SearchAddressRequest>,
) -> Result<Json<Vec<AddressResponse>>, AppError> {
    let controller = AddressController::new(state.pool.clone());
    let response = controller.search(request).await?;
    Ok(Json(response))
}

async fn geocode_address(
    State(state): State<AppState>,
    Json(request): Json<GeocodeRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let controller = AddressController::new(state.pool.clone());
    let response = controller.geocode_address(request.address).await?;
    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
struct GeocodeRequest {
    address: String,
}

#[derive(Debug, Deserialize)]
struct UpdateDetailsRequest {
    door_codes: Option<String>,
    mailbox_access: Option<bool>,
    access_instructions: Option<String>,
}

async fn update_address_details(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateDetailsRequest>,
) -> Result<Json<ApiResponse<AddressResponse>>, AppError> {
    let controller = AddressController::new(state.pool.clone());
    let response = controller.update_details(
        id,
        request.door_codes,
        request.mailbox_access,
        request.access_instructions,
    ).await?;
    Ok(Json(response))
}

async fn delete_address(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let controller = AddressController::new(state.pool.clone());
    controller.delete(id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Dirección eliminada exitosamente"
    })))
}
