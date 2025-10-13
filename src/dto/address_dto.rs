use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Request para guardar/actualizar una dirección
#[derive(Debug, Deserialize)]
pub struct SaveAddressRequest {
    pub route_id: Uuid,
    pub address: String,
    pub postal_code: Option<String>,
    pub door_codes: Option<String>,
    pub mailbox_access: Option<bool>,
    pub access_instructions: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

// Response de dirección
#[derive(Debug, Serialize)]
pub struct AddressResponse {
    pub id: Uuid,
    pub route_id: Uuid,
    pub address: String,
    pub postal_code: Option<String>,
    pub door_codes: Option<String>,
    pub mailbox_access: bool,
    pub access_instructions: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub created_at: DateTime<Utc>,
}

// Request para buscar direcciones
#[derive(Debug, Deserialize)]
pub struct SearchAddressRequest {
    pub address: Option<String>,
    pub postal_code: Option<String>,
}
