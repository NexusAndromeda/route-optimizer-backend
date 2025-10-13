//! Modelo de Route
//! 
//! Modelo simplificado para routes - solo para referencias de tournées

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Route principal - mapea exactamente a la tabla routes del schema simplificado
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Route {
    pub id: Uuid,
    pub company_id: Uuid,
    pub vehicle_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl Route {
    pub fn new(company_id: Uuid, vehicle_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            company_id,
            vehicle_id,
            created_at: Utc::now(),
        }
    }
}
