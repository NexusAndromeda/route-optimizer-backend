//! Modelo de Route
//! 
//! Este módulo contiene el struct Route y sus variantes para CRUD operations.
//! Mapea exactamente al schema PostgreSQL con primary key 'id'.

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use validator::Validate;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;

/// Estado de la ruta - mapea al ENUM route_status
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "route_status", rename_all = "lowercase")]
pub enum RouteStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
    Paused,
}

/// Origen de la ruta - mapea al campo route_origin
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RouteOrigin {
    Manual,
    ApiSync,
    Webhook,
}

/// Route principal - mapea exactamente a la tabla routes del schema simplificado
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Route {
    pub id: Uuid,
    pub company_id: Uuid,
    pub vehicle_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Request para crear una nueva ruta
#[derive(Debug, Deserialize, Validate)]
pub struct CreateRouteRequest {
    pub driver_id: String,
    pub vehicle_id: String,
    
    #[validate(length(min = 3, max = 50))]
    pub route_number: Option<String>,
    
    #[validate(length(min = 5, max = 500))]
    pub start_location: Option<String>,
    
    #[validate(length(min = 5, max = 500))]
    pub end_location: Option<String>,
    
    pub start_mileage: Option<Decimal>,
    
    #[validate(range(min = 0))]
    pub estimated_duration_minutes: Option<i32>,
    
    pub route_origin: Option<String>,
    pub external_route_id: Option<String>,
}

/// Request para actualizar una ruta existente
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRouteRequest {
    pub route_status: Option<String>,
    
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

/// Request para iniciar una ruta
#[derive(Debug, Deserialize, Validate)]
pub struct StartRouteRequest {
    pub start_mileage: Decimal,
    
    pub pre_inspection_notes: Option<String>,
    pub pre_inspection_photos: Option<Vec<String>>,
}

/// Request para finalizar una ruta
#[derive(Debug, Deserialize, Validate)]
pub struct EndRouteRequest {
    pub end_mileage: Decimal,
    
    pub fuel_consumed: Option<Decimal>,
    
    pub fuel_cost: Option<Decimal>,
    
    pub post_inspection_notes: Option<String>,
    pub post_inspection_photos: Option<Vec<String>>,
}

/// Response de ruta para la API - simplificado
#[derive(Debug, Serialize)]
pub struct RouteResponse {
    pub id: String,
    pub company_id: String,
    pub vehicle_id: Option<String>,
    pub created_at: String,
}

/// Response de ruta para listados - simplificado
#[derive(Debug, Serialize)]
pub struct RouteListResponse {
    pub id: String,
    pub company_id: String,
    pub vehicle_id: Option<String>,
    pub created_at: String,
}

/// Filtros para búsqueda de rutas
#[derive(Debug, Deserialize)]
pub struct RouteFilters {
    pub route_status: Option<String>,
    pub driver_id: Option<String>,
    pub vehicle_id: Option<String>,
    pub route_date_from: Option<String>,
    pub route_date_to: Option<String>,
    pub route_origin: Option<String>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<Route> for RouteResponse {
    fn from(route: Route) -> Self {
        Self {
            id: route.id.to_string(),
            company_id: route.company_id.to_string(),
            vehicle_id: route.vehicle_id.map(|v| v.to_string()),
            created_at: route.created_at.to_rfc3339(),
        }
    }
}

impl From<Route> for RouteListResponse {
    fn from(route: Route) -> Self {
        Self {
            id: route.id.to_string(),
            company_id: route.company_id.to_string(),
            vehicle_id: route.vehicle_id.map(|v| v.to_string()),
            created_at: route.created_at.to_rfc3339(),
        }
    }
}
