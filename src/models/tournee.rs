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

/// Route principal - mapea exactamente a la tabla routes del schema simplificado
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tournee {
    pub id: Uuid,
    pub company_id: Uuid,
    pub vehicle_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
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

/// Response de tournée para la API - simplificado
#[derive(Debug, Serialize)]
pub struct TourneeResponse {
    pub id: String,
    pub company_id: String,
    pub vehicle_id: Option<String>,
    pub created_at: String,
}

/// Response de tournée para listados - simplificado
#[derive(Debug, Serialize)]
pub struct TourneeListResponse {
    pub id: String,
    pub company_id: String,
    pub vehicle_id: Option<String>,
    pub created_at: String,
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
            company_id: tournee.company_id.to_string(),
            vehicle_id: tournee.vehicle_id.map(|v| v.to_string()),
            created_at: tournee.created_at.to_rfc3339(),
        }
    }
}

impl From<Tournee> for TourneeListResponse {
    fn from(tournee: Tournee) -> Self {
        Self {
            id: tournee.id.to_string(),
            company_id: tournee.company_id.to_string(),
            vehicle_id: tournee.vehicle_id.map(|v| v.to_string()),
            created_at: tournee.created_at.to_rfc3339(),
        }
    }
}
