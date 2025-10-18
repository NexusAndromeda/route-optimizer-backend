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
    /// Genera una clave de búsqueda normalizada para matching
    pub fn search_key(&self) -> String {
        let number = self.street_number.as_deref().unwrap_or("").trim();
        let street = normalize_street(&self.street_name);
        let postcode = self.postcode.trim();
        
        format!("{} {} {}", number, street, postcode)
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }
    
    pub fn from_search(street_name: String, postcode: String) -> AddressSearch {
        AddressSearch { street_name, postcode }
    }
}

/// Normaliza el nombre de una calle para matching robusto
fn normalize_street(street: &str) -> String {
    street
        .to_uppercase()
        // Normalizar acentos franceses
        .replace("É", "E")
        .replace("È", "E")
        .replace("Ê", "E")
        .replace("Ë", "E")
        .replace("À", "A")
        .replace("Â", "A")
        .replace("Ô", "O")
        .replace("Ù", "U")
        .replace("Û", "U")
        .replace("Ç", "C")
        .replace("Î", "I")
        .replace("Ï", "I")
        // Quitar puntuación
        .replace(",", "")
        .replace(".", "")
        .replace(";", "")
        .replace(":", "")
        // Limpiar espacios extra
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColisPriveAddress {
    pub num_voie: Option<String>,
    pub libelle_voie: String,
    pub code_postal: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl ColisPriveAddress {
    /// Genera una clave de búsqueda normalizada para matching con BD
    pub fn search_key(&self) -> String {
        let number = self.num_voie.as_deref().unwrap_or("").trim();
        let street = normalize_street(&self.libelle_voie);
        let postcode = self.code_postal.trim();
        
        format!("{} {} {}", number, street, postcode)
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }
}

impl From<ColisPriveAddress> for AddressSearch {
    fn from(addr: ColisPriveAddress) -> Self {
        Self {
            street_name: addr.libelle_voie,
            postcode: addr.code_postal,
        }
    }
}
