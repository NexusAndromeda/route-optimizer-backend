use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Address {
    pub id: Uuid,
    pub company_id: Option<Uuid>,
    
    // Datos oficiales de la API francesa
    pub official_label: String,
    pub street_name: String,
    pub street_number: Option<String>,
    pub postcode: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    
    // Datos del chofer (compartidos para esta dirección)
    pub door_code: Option<String>,
    pub has_mailbox_access: bool,
    pub driver_notes: Option<String>,
    
    // Metadata
    pub last_updated_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressSearch {
    pub street_name: String,
    pub postcode: String,
}

impl Address {
    pub fn search_key(&self) -> String {
        format!("{} {}", self.street_name, self.postcode)
    }
    
    pub fn from_search(street_name: String, postcode: String) -> AddressSearch {
        AddressSearch { street_name, postcode }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColisPriveAddress {
    pub num_voie: Option<String>,
    pub libelle_voie: String,
    pub code_postal: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl From<ColisPriveAddress> for AddressSearch {
    fn from(addr: ColisPriveAddress) -> Self {
        Self {
            street_name: addr.libelle_voie,
            postcode: addr.code_postal,
        }
    }
}
