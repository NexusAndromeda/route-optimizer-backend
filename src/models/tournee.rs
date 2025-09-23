//! Modelo de Tournee
//! 
//! Este módulo contiene el struct Tournee y sus variantes para CRUD operations.
//! Mapea exactamente al schema PostgreSQL con primary key 'id'.

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use validator::Validate;
use chrono::{DateTime, Utc, NaiveDate};
use uuid::Uuid;
use rust_decimal::Decimal;

/// Estado de la tournée - mapea al ENUM tournee_status
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "tournee_status", rename_all = "lowercase")]
pub enum TourneeStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
    Paused,
}

/// Origen de la tournée - mapea al campo tournee_origin
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TourneeOrigin {
    Manual,
    ApiSync,
    Webhook,
}

/// Tournee principal - mapea exactamente a la tabla tournees
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tournee {
    pub id: Uuid,
    pub company_id: Uuid,
    pub driver_id: Uuid,
    pub vehicle_id: Uuid,
    
    // Información de la ruta
    pub tournee_date: NaiveDate,
    pub tournee_number: Option<String>,
    pub start_location: Option<String>,
    pub end_location: Option<String>,
    
    // Estado operativo
    pub tournee_status: TourneeStatus,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    
    // Métricas de kilometraje y combustible
    pub start_mileage: Option<Decimal>,
    pub end_mileage: Option<Decimal>,
    pub total_distance: Option<Decimal>,
    pub fuel_consumed: Option<Decimal>,
    pub fuel_cost: Option<Decimal>,
    
    // Inspecciones
    pub pre_inspection_notes: Option<String>,
    pub post_inspection_notes: Option<String>,
    pub pre_inspection_photos: Option<Vec<String>>,
    pub post_inspection_photos: Option<Vec<String>>,
    
    // Optimización de ruta
    pub route_optimization_score: Option<Decimal>,
    pub estimated_duration_minutes: Option<i32>,
    pub actual_duration_minutes: Option<i32>,
    
    // Ruta y condiciones
    pub route_coordinates: Option<Vec<String>>,
    pub traffic_conditions: Option<serde_json::Value>,
    pub weather_conditions: Option<serde_json::Value>,
    
    // Origen de la tournée
    pub tournee_origin: Option<String>,
    pub external_tournee_id: Option<String>,
    pub integration_id: Option<Uuid>,
    
    // Metadatos
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Request para crear una nueva tournée
#[derive(Debug, Deserialize, Validate)]
pub struct CreateTourneeRequest {
    pub driver_id: String,
    pub vehicle_id: String,
    
    #[validate(length(min = 3, max = 50))]
    pub tournee_number: Option<String>,
    
    #[validate(length(min = 5, max = 500))]
    pub start_location: Option<String>,
    
    #[validate(length(min = 5, max = 500))]
    pub end_location: Option<String>,
    
    pub start_mileage: Option<Decimal>,
    
    #[validate(range(min = 0))]
    pub estimated_duration_minutes: Option<i32>,
    
    pub tournee_origin: Option<String>,
    pub external_tournee_id: Option<String>,
}

/// Request para actualizar una tournée existente
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTourneeRequest {
    pub tournee_status: Option<String>,
    
    #[validate(length(min = 5, max = 500))]
    pub start_location: Option<String>,
    
    #[validate(length(min = 5, max = 500))]
    pub end_location: Option<String>,
    
    pub start_mileage: Option<Decimal>,
    
    pub end_mileage: Option<Decimal>,
    
    pub fuel_consumed: Option<Decimal>,
    
    pub fuel_cost: Option<Decimal>,
    
    pub pre_inspection_notes: Option<String>,
    pub post_inspection_notes: Option<String>,
    pub route_optimization_score: Option<Decimal>,
    pub actual_duration_minutes: Option<i32>,
}

/// Request para iniciar una tournée
#[derive(Debug, Deserialize, Validate)]
pub struct StartTourneeRequest {
    pub start_mileage: Decimal,
    
    pub pre_inspection_notes: Option<String>,
    pub pre_inspection_photos: Option<Vec<String>>,
}

/// Request para finalizar una tournée
#[derive(Debug, Deserialize, Validate)]
pub struct EndTourneeRequest {
    pub end_mileage: Decimal,
    
    pub fuel_consumed: Option<Decimal>,
    
    pub fuel_cost: Option<Decimal>,
    
    pub post_inspection_notes: Option<String>,
    pub post_inspection_photos: Option<Vec<String>>,
}

/// Response de tournée para la API
#[derive(Debug, Serialize)]
pub struct TourneeResponse {
    pub id: String,
    pub tournee_number: Option<String>,
    pub tournee_date: String,
    pub start_location: Option<String>,
    pub end_location: Option<String>,
    pub tournee_status: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub start_mileage: Option<String>,
    pub end_mileage: Option<String>,
    pub total_distance: Option<String>,
    pub fuel_consumed: Option<String>,
    pub fuel_cost: Option<String>,
    pub route_optimization_score: Option<String>,
    pub estimated_duration_minutes: Option<i32>,
    pub actual_duration_minutes: Option<i32>,
    pub driver_id: String,
    pub vehicle_id: String,
    pub company_id: String,
    pub tournee_origin: Option<String>,
    pub external_tournee_id: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Response de tournée para listados
#[derive(Debug, Serialize)]
pub struct TourneeListResponse {
    pub id: String,
    pub tournee_number: Option<String>,
    pub tournee_date: String,
    pub tournee_status: String,
    pub start_location: Option<String>,
    pub end_location: Option<String>,
    pub total_distance: Option<String>,
    pub fuel_consumed: Option<String>,
    pub fuel_cost: Option<String>,
    pub driver_id: String,
    pub vehicle_id: String,
    pub created_at: Option<String>,
}

/// Filtros para búsqueda de tournées
#[derive(Debug, Deserialize)]
pub struct TourneeFilters {
    pub tournee_status: Option<String>,
    pub driver_id: Option<String>,
    pub vehicle_id: Option<String>,
    pub tournee_date_from: Option<String>,
    pub tournee_date_to: Option<String>,
    pub tournee_origin: Option<String>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<Tournee> for TourneeResponse {
    fn from(tournee: Tournee) -> Self {
        Self {
            id: tournee.id.to_string(),
            tournee_number: tournee.tournee_number,
            tournee_date: tournee.tournee_date.to_string(),
            start_location: tournee.start_location,
            end_location: tournee.end_location,
            tournee_status: format!("{:?}", tournee.tournee_status).to_lowercase(),
            start_time: tournee.start_time.map(|dt| dt.to_rfc3339()),
            end_time: tournee.end_time.map(|dt| dt.to_rfc3339()),
            start_mileage: tournee.start_mileage.map(|m| m.to_string()),
            end_mileage: tournee.end_mileage.map(|m| m.to_string()),
            total_distance: tournee.total_distance.map(|d| d.to_string()),
            fuel_consumed: tournee.fuel_consumed.map(|f| f.to_string()),
            fuel_cost: tournee.fuel_cost.map(|f| f.to_string()),
            route_optimization_score: tournee.route_optimization_score.map(|s| s.to_string()),
            estimated_duration_minutes: tournee.estimated_duration_minutes,
            actual_duration_minutes: tournee.actual_duration_minutes,
            driver_id: tournee.driver_id.to_string(),
            vehicle_id: tournee.vehicle_id.to_string(),
            company_id: tournee.company_id.to_string(),
            tournee_origin: tournee.tournee_origin,
            external_tournee_id: tournee.external_tournee_id,
            created_at: tournee.created_at.map(|dt| dt.to_rfc3339()),
            updated_at: tournee.updated_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

impl From<Tournee> for TourneeListResponse {
    fn from(tournee: Tournee) -> Self {
        Self {
            id: tournee.id.to_string(),
            tournee_number: tournee.tournee_number,
            tournee_date: tournee.tournee_date.to_string(),
            tournee_status: format!("{:?}", tournee.tournee_status).to_lowercase(),
            start_location: tournee.start_location,
            end_location: tournee.end_location,
            total_distance: tournee.total_distance.map(|d| d.to_string()),
            fuel_consumed: tournee.fuel_consumed.map(|f| f.to_string()),
            fuel_cost: tournee.fuel_cost.map(|f| f.to_string()),
            driver_id: tournee.driver_id.to_string(),
            vehicle_id: tournee.vehicle_id.to_string(),
            created_at: tournee.created_at.map(|dt| dt.to_rfc3339()),
        }
    }
}
