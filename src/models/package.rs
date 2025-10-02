//! Modelo de Package
//! 
//! Este módulo contiene el struct Package y sus variantes para CRUD operations.
//! Mapea exactamente al schema PostgreSQL con primary key 'id'.

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use validator::Validate;
use chrono::{DateTime, Utc, NaiveDate, NaiveTime};
use uuid::Uuid;
use rust_decimal::Decimal;

/// Estado de entrega - mapea al ENUM delivery_status
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "delivery_status", rename_all = "lowercase")]
pub enum DeliveryStatus {
    Pending,
    InTransit,
    OutForDelivery,
    Delivered,
    Failed,
    Returned,
    Cancelled,
}

/// Razón de fallo en entrega - mapea al ENUM delivery_failure_reason
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "delivery_failure_reason", rename_all = "lowercase")]
pub enum DeliveryFailureReason {
    RecipientNotHome,
    WrongAddress,
    PackageDamaged,
    RefusedDelivery,
    SecurityRestriction,
    WeatherConditions,
    VehicleBreakdown,
    DriverEmergency,
}

/// Origen del paquete - mapea al campo package_origin
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PackageOrigin {
    Manual,
    ApiSync,
    Webhook,
}

/// Package simplificado - NO EXISTE EN EL SCHEMA SIMPLIFICADO
/// Este modelo se mantiene para compatibilidad pero no se usa en el schema actual
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Package {
    pub id: Uuid,
    pub company_id: Uuid,
    pub tournee_id: Uuid,
    pub tracking_number: String,
    pub delivery_address: String,
    pub delivery_status: DeliveryStatus,
    pub created_at: DateTime<Utc>,
}

/// Tipo Point para PostGIS - mapea al tipo POINT
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "point")]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Request para crear un nuevo paquete
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePackageRequest {
    pub tournee_id: String,
    
    #[validate(length(min = 5, max = 100))]
    pub tracking_number: String,
    
    pub external_tracking_number: Option<String>,
    pub package_origin: Option<String>,
    pub external_package_id: Option<String>,
    pub package_type: Option<String>,
    
    pub package_weight: Option<Decimal>,
    
    pub package_dimensions: Option<String>,
    
    #[validate(length(min = 5, max = 500))]
    pub delivery_address: String,
    
    pub recipient_name: Option<String>,
    pub recipient_phone: Option<String>,
    pub delivery_instructions: Option<String>,
    pub signature_required: Option<bool>,
}

/// Request para actualizar un paquete existente
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePackageRequest {
    pub delivery_status: Option<String>,
    pub delivery_date: Option<String>,
    pub delivery_time: Option<String>,
    
    #[validate(length(min = 5, max = 500))]
    pub delivery_address: Option<String>,
    
    pub recipient_name: Option<String>,
    pub recipient_phone: Option<String>,
    pub delivery_instructions: Option<String>,
    pub failure_reason: Option<String>,
    pub failure_notes: Option<String>,
    pub reschedule_date: Option<String>,
    pub driver_notes: Option<String>,
    pub package_condition: Option<String>,
}

/// Request para marcar paquete como entregado
#[derive(Debug, Deserialize, Validate)]
pub struct MarkDeliveredRequest {
    pub delivery_photo: Option<String>,
    pub signature_image: Option<String>,
    pub signature_photo: Option<String>,
    pub delivery_duration_minutes: Option<i32>,
    pub driver_notes: Option<String>,
    pub package_condition: Option<String>,
}

/// Request para marcar paquete como fallido
#[derive(Debug, Deserialize, Validate)]
pub struct MarkFailedRequest {
    pub failure_reason: String,
    pub failure_notes: Option<String>,
    pub reschedule_date: Option<String>,
    pub driver_notes: Option<String>,
}

/// Response de paquete para la API - simplificado
#[derive(Debug, Serialize)]
pub struct PackageResponse {
    pub id: String,
    pub tracking_number: String,
    pub delivery_address: String,
    pub delivery_status: String,
    pub tournee_id: String,
    pub company_id: String,
    pub created_at: String,
}

/// Response de paquete para listados - simplificado
#[derive(Debug, Serialize)]
pub struct PackageListResponse {
    pub id: String,
    pub tracking_number: String,
    pub delivery_address: String,
    pub delivery_status: String,
    pub tournee_id: String,
    pub company_id: String,
    pub created_at: String,
}

/// Filtros para búsqueda de paquetes
#[derive(Debug, Deserialize)]
pub struct PackageFilters {
    pub delivery_status: Option<String>,
    pub tournee_id: Option<String>,
    pub tracking_number: Option<String>,
    pub external_tracking_number: Option<String>,
    pub package_origin: Option<String>,
    pub delivery_date_from: Option<String>,
    pub delivery_date_to: Option<String>,
    pub failure_reason: Option<String>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<Package> for PackageResponse {
    fn from(package: Package) -> Self {
        Self {
            id: package.id.to_string(),
            tracking_number: package.tracking_number,
            delivery_address: package.delivery_address,
            delivery_status: format!("{:?}", package.delivery_status).to_lowercase(),
            tournee_id: package.tournee_id.to_string(),
            company_id: package.company_id.to_string(),
            created_at: package.created_at.to_rfc3339(),
        }
    }
}

impl From<Package> for PackageListResponse {
    fn from(package: Package) -> Self {
        Self {
            id: package.id.to_string(),
            tracking_number: package.tracking_number,
            delivery_address: package.delivery_address,
            delivery_status: format!("{:?}", package.delivery_status).to_lowercase(),
            tournee_id: package.tournee_id.to_string(),
            company_id: package.company_id.to_string(),
            created_at: package.created_at.to_rfc3339(),
        }
    }
}
