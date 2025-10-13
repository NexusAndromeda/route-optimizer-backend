//! Modelo de Vehicle
//! 
//! Modelo simplificado para vehicles

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;

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

impl Vehicle {
    pub fn new(
        company_id: Uuid,
        license_plate: String,
        brand: Option<String>,
        model: Option<String>,
        fuel_type: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            company_id,
            license_plate,
            brand,
            model,
            vehicle_status: "active".to_string(),
            current_mileage: Decimal::ZERO,
            fuel_type,
            created_at: Utc::now(),
        }
    }
}
