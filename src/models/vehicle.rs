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

/// Vehicle principal - mapea exactamente a la tabla vehicles
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Vehicle {
    pub id: Uuid,
    pub company_id: Uuid,
    pub license_plate: String,
    pub brand: String,
    pub model: String,
    pub year: Option<i32>,
    pub color: Option<String>,
    
    // Estado operativo
    pub vehicle_status: VehicleStatus,
    pub current_mileage: Decimal,
    pub fuel_type: String,
    pub fuel_capacity: Option<Decimal>,
    pub weekly_fuel_allocation: Option<Decimal>,
    
    // Métricas de daños
    pub total_damage_cost: Decimal,
    pub damage_incidents_count: i32,
    
    // Información técnica
    pub vin: Option<String>,
    pub engine_size: Option<String>,
    pub transmission: Option<String>,
    
    // Metadatos
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
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

/// Response de vehículo para la API
#[derive(Debug, Serialize)]
pub struct VehicleResponse {
    pub id: String,
    pub license_plate: String,
    pub brand: String,
    pub model: String,
    pub year: Option<i32>,
    pub color: Option<String>,
    pub vehicle_status: String,
    pub current_mileage: String,
    pub fuel_type: String,
    pub fuel_capacity: Option<String>,
    pub weekly_fuel_allocation: Option<String>,
    pub total_damage_cost: String,
    pub damage_incidents_count: i32,
    pub vin: Option<String>,
    pub engine_size: Option<String>,
    pub transmission: Option<String>,
    pub company_id: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Response de vehículo para listados
#[derive(Debug, Serialize)]
pub struct VehicleListResponse {
    pub id: String,
    pub license_plate: String,
    pub brand: String,
    pub model: String,
    pub year: Option<i32>,
    pub color: Option<String>,
    pub vehicle_status: String,
    pub current_mileage: String,
    pub fuel_type: String,
    pub total_damage_cost: String,
    pub damage_incidents_count: i32,
    pub created_at: Option<String>,
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
            year: vehicle.year,
            color: vehicle.color,
            vehicle_status: format!("{:?}", vehicle.vehicle_status).to_lowercase(),
            current_mileage: vehicle.current_mileage.to_string(),
            fuel_type: vehicle.fuel_type,
            fuel_capacity: vehicle.fuel_capacity.map(|f| f.to_string()),
            weekly_fuel_allocation: vehicle.weekly_fuel_allocation.map(|f| f.to_string()),
            total_damage_cost: vehicle.total_damage_cost.to_string(),
            damage_incidents_count: vehicle.damage_incidents_count,
            vin: vehicle.vin,
            engine_size: vehicle.engine_size,
            transmission: vehicle.transmission,
            company_id: vehicle.company_id.to_string(),
            created_at: vehicle.created_at.map(|dt| dt.to_rfc3339()),
            updated_at: vehicle.updated_at.map(|dt| dt.to_rfc3339()),
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
            year: vehicle.year,
            color: vehicle.color,
            vehicle_status: format!("{:?}", vehicle.vehicle_status).to_lowercase(),
            current_mileage: vehicle.current_mileage.to_string(),
            fuel_type: vehicle.fuel_type,
            total_damage_cost: vehicle.total_damage_cost.to_string(),
            damage_incidents_count: vehicle.damage_incidents_count,
            created_at: vehicle.created_at.map(|dt| dt.to_rfc3339()),
        }
    }
}
