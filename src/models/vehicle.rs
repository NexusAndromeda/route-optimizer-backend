//! Modelo de Vehicle
//! 
//! Este módulo contiene el struct Vehicle y sus variantes para CRUD operations.
//! Mapea exactamente al schema PostgreSQL con primary key 'id'.

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use validator::Validate;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;

/// Estado del vehículo - mapea al ENUM vehicle_status
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "vehicle_status", rename_all = "lowercase")]
pub enum VehicleStatus {
    Active,
    Maintenance,
    OutOfService,
    Retired,
}

/// Vehicle principal - mapea exactamente a la tabla vehicles del schema simplificado
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Vehicle {
    pub id: Uuid,
    pub company_id: Uuid,
    pub license_plate: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub vehicle_status: String,
    pub current_mileage: Decimal,
    pub fuel_type: String,
    pub created_at: DateTime<Utc>,
}

/// Request para crear un nuevo vehículo
#[derive(Debug, Deserialize, Validate)]
pub struct CreateVehicleRequest {
    #[validate(length(min = 5, max = 20))]
    pub license_plate: String,
    
    #[validate(length(min = 2, max = 100))]
    pub brand: String,
    
    #[validate(length(min = 2, max = 100))]
    pub model: String,
    
    #[validate(range(min = 1900, max = 2030))]
    pub year: Option<i32>,
    
    #[validate(length(min = 2, max = 50))]
    pub color: Option<String>,
    
    #[validate(length(min = 2, max = 20))]
    pub fuel_type: String,
    
    pub fuel_capacity: Option<Decimal>,
    
    pub weekly_fuel_allocation: Option<Decimal>,
    
    pub vin: Option<String>,
    pub engine_size: Option<String>,
    pub transmission: Option<String>,
}

/// Request para actualizar un vehículo existente
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateVehicleRequest {
    #[validate(length(min = 5, max = 20))]
    pub license_plate: Option<String>,
    
    #[validate(length(min = 2, max = 100))]
    pub brand: Option<String>,
    
    #[validate(length(min = 2, max = 100))]
    pub model: Option<String>,
    
    #[validate(range(min = 1900, max = 2030))]
    pub year: Option<i32>,
    
    #[validate(length(min = 2, max = 50))]
    pub color: Option<String>,
    
    pub vehicle_status: Option<String>,
    
    pub current_mileage: Option<Decimal>,
    
    #[validate(length(min = 2, max = 20))]
    pub fuel_type: Option<String>,
    
    pub fuel_capacity: Option<Decimal>,
    
    pub weekly_fuel_allocation: Option<Decimal>,
    
    pub vin: Option<String>,
    pub engine_size: Option<String>,
    pub transmission: Option<String>,
}

/// Response de vehículo para la API - simplificado
#[derive(Debug, Serialize)]
pub struct VehicleResponse {
    pub id: String,
    pub license_plate: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub vehicle_status: String,
    pub current_mileage: String,
    pub fuel_type: String,
    pub company_id: String,
    pub created_at: String,
}

/// Response de vehículo para listados - simplificado
#[derive(Debug, Serialize)]
pub struct VehicleListResponse {
    pub id: String,
    pub license_plate: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub vehicle_status: String,
    pub current_mileage: String,
    pub fuel_type: String,
    pub created_at: String,
}

/// Filtros para búsqueda de vehículos
#[derive(Debug, Deserialize)]
pub struct VehicleFilters {
    pub vehicle_status: Option<String>,
    pub fuel_type: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub year_from: Option<i32>,
    pub year_to: Option<i32>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Request para actualizar kilometraje
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMileageRequest {
    pub current_mileage: Decimal,
}

/// Request para actualizar estado del vehículo
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateVehicleStatusRequest {
    pub vehicle_status: String,
    pub notes: Option<String>,
}

impl From<Vehicle> for VehicleResponse {
    fn from(vehicle: Vehicle) -> Self {
        Self {
            id: vehicle.id.to_string(),
            license_plate: vehicle.license_plate,
            brand: vehicle.brand,
            model: vehicle.model,
            vehicle_status: vehicle.vehicle_status,
            current_mileage: vehicle.current_mileage.to_string(),
            fuel_type: vehicle.fuel_type,
            company_id: vehicle.company_id.to_string(),
            created_at: vehicle.created_at.to_rfc3339(),
        }
    }
}

impl From<Vehicle> for VehicleListResponse {
    fn from(vehicle: Vehicle) -> Self {
        Self {
            id: vehicle.id.to_string(),
            license_plate: vehicle.license_plate,
            brand: vehicle.brand,
            model: vehicle.model,
            vehicle_status: vehicle.vehicle_status,
            current_mileage: vehicle.current_mileage.to_string(),
            fuel_type: vehicle.fuel_type,
            created_at: vehicle.created_at.to_rfc3339(),
        }
    }
}
