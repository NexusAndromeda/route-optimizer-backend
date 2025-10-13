use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Request para crear un vehículo
#[derive(Debug, Deserialize)]
pub struct CreateVehicleRequest {
    pub license_plate: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub fuel_type: Option<String>,
    pub current_mileage: Option<f64>,
}

// Request para actualizar un vehículo
#[derive(Debug, Deserialize)]
pub struct UpdateVehicleRequest {
    pub license_plate: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub vehicle_status: Option<String>,
    pub current_mileage: Option<f64>,
    pub fuel_type: Option<String>,
}

// Response de vehículo
#[derive(Debug, Serialize)]
pub struct VehicleResponse {
    pub id: Uuid,
    pub company_id: Uuid,
    pub license_plate: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub vehicle_status: String,
    pub current_mileage: f64,
    pub fuel_type: String,
    pub created_at: DateTime<Utc>,
}
