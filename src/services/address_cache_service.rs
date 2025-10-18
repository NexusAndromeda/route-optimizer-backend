use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use crate::services::geocoding_service::{GeocodingService, GeocodingResponse};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CachedAddress {
    pub id: String,
    pub official_label: String,
    pub street_name: String,
    pub street_number: Option<String>,
    pub postcode: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub door_code: Option<String>,
    pub has_mailbox_access: bool,
    pub driver_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressCacheResult {
    pub found: bool,
    pub address: Option<CachedAddress>,
    pub source: AddressSource,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AddressSource {
    Database,
    Mapbox,
    NotFound,
}

pub struct AddressCacheService {
    pool: PgPool,
    geocoding_service: GeocodingService,
    // Cache en memoria para evitar consultas repetidas
    memory_cache: HashMap<String, CachedAddress>,
}

impl AddressCacheService {
    pub fn new(pool: PgPool, geocoding_service: GeocodingService) -> Self {
        Self {
            pool,
            geocoding_service,
            memory_cache: HashMap::new(),
        }
    }

    /// Buscar dirección en caché (BD + memoria) antes de usar Mapbox
    pub async fn find_or_geocode_address(
        &mut self,
        address: &str,
        company_id: &str,
    ) -> Result<AddressCacheResult> {
        log::info!("🔍 Buscando dirección en caché: '{}' para empresa: '{}'", address, company_id);

        // 1. Buscar en caché de memoria primero (más rápido)
        if let Some(cached) = self.memory_cache.get(address) {
            log::info!("✅ Dirección encontrada en caché de memoria: {}", address);
            return Ok(AddressCacheResult {
                found: true,
                address: Some(cached.clone()),
                source: AddressSource::Database,
            });
        }

        // 2. Buscar en base de datos
        if let Some(cached) = self.find_in_database(address, company_id).await? {
            log::info!("✅ Dirección encontrada en base de datos: {}", address);
            
            // Guardar en caché de memoria
            self.memory_cache.insert(address.to_string(), cached.clone());
            
            return Ok(AddressCacheResult {
                found: true,
                address: Some(cached),
                source: AddressSource::Database,
            });
        }

        // 3. No encontrada en caché, usar Mapbox
        log::info!("❌ Dirección no encontrada en caché, usando Mapbox: {}", address);
        
        match self.geocoding_service.geocode_address(address).await {
            Ok(response) => {
                if response.success {
                    // Guardar en base de datos para futuras consultas
                    if let Some(saved_address) = self.save_to_database(
                        address,
                        &response,
                        company_id,
                    ).await? {
                        // Guardar en caché de memoria
                        self.memory_cache.insert(address.to_string(), saved_address.clone());
                        
                        Ok(AddressCacheResult {
                            found: true,
                            address: Some(saved_address),
                            source: AddressSource::Mapbox,
                        })
                    } else {
                        Ok(AddressCacheResult {
                            found: false,
                            address: None,
                            source: AddressSource::NotFound,
                        })
                    }
                } else {
                    Ok(AddressCacheResult {
                        found: false,
                        address: None,
                        source: AddressSource::NotFound,
                    })
                }
            }
            Err(e) => {
                log::error!("❌ Error en geocodificación: {}", e);
                Ok(AddressCacheResult {
                    found: false,
                    address: None,
                    source: AddressSource::NotFound,
                })
            }
        }
    }

    /// Buscar dirección en base de datos
    async fn find_in_database(
        &self,
        address: &str,
        company_id: &str,
    ) -> Result<Option<CachedAddress>> {
        // Buscar por coincidencia exacta primero
        if let Some(cached) = self.find_exact_match(address, company_id).await? {
            return Ok(Some(cached));
        }

        // Buscar por coincidencia parcial (calle + número)
        if let Some(cached) = self.find_partial_match(address, company_id).await? {
            return Ok(Some(cached));
        }

        Ok(None)
    }

    /// Búsqueda exacta por official_label
    async fn find_exact_match(
        &self,
        address: &str,
        company_id: &str,
    ) -> Result<Option<CachedAddress>> {
        let query = r#"
            SELECT 
                id,
                official_label,
                street_name,
                street_number,
                postcode,
                city,
                ST_Y(coordinates) as latitude,
                ST_X(coordinates) as longitude,
                door_code,
                has_mailbox_access,
                driver_notes
            FROM addresses 
            WHERE company_id = $1 
            AND LOWER(official_label) = LOWER($2)
            LIMIT 1
        "#;

        let row = sqlx::query(query)
            .bind(company_id)
            .bind(address)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(CachedAddress {
                id: row.get("id"),
                official_label: row.get("official_label"),
                street_name: row.get("street_name"),
                street_number: row.get("street_number"),
                postcode: row.get("postcode"),
                city: row.get("city"),
                latitude: row.get("latitude"),
                longitude: row.get("longitude"),
                door_code: row.get("door_code"),
                has_mailbox_access: row.get("has_mailbox_access"),
                driver_notes: row.get("driver_notes"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Búsqueda parcial por calle y número
    async fn find_partial_match(
        &self,
        address: &str,
        company_id: &str,
    ) -> Result<Option<CachedAddress>> {
        // Extraer número y calle de la dirección
        let (street_number, street_name) = self.extract_street_components(address);
        
        if street_number.is_none() || street_name.is_empty() {
            return Ok(None);
        }

        let query = r#"
            SELECT 
                id,
                official_label,
                street_name,
                street_number,
                postcode,
                city,
                ST_Y(coordinates) as latitude,
                ST_X(coordinates) as longitude,
                door_code,
                has_mailbox_access,
                driver_notes
            FROM addresses 
            WHERE company_id = $1 
            AND LOWER(street_name) = LOWER($2)
            AND street_number = $3
            LIMIT 1
        "#;

        let row = sqlx::query(query)
            .bind(company_id)
            .bind(&street_name)
            .bind(&street_number.unwrap())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(CachedAddress {
                id: row.get("id"),
                official_label: row.get("official_label"),
                street_name: row.get("street_name"),
                street_number: row.get("street_number"),
                postcode: row.get("postcode"),
                city: row.get("city"),
                latitude: row.get("latitude"),
                longitude: row.get("longitude"),
                door_code: row.get("door_code"),
                has_mailbox_access: row.get("has_mailbox_access"),
                driver_notes: row.get("driver_notes"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Guardar nueva dirección en base de datos
    async fn save_to_database(
        &self,
        original_address: &str,
        geocoding_response: &GeocodingResponse,
        company_id: &str,
    ) -> Result<Option<CachedAddress>> {
        let latitude = geocoding_response.latitude.unwrap_or(0.0);
        let longitude = geocoding_response.longitude.unwrap_or(0.0);
        let formatted_address = geocoding_response.formatted_address.clone()
            .unwrap_or_else(|| original_address.to_string());

        // Extraer componentes de la dirección formateada
        let (street_number, street_name) = self.extract_street_components(&formatted_address);
        let (postcode, city) = self.extract_postcode_city(&formatted_address);

        let query = r#"
            INSERT INTO addresses (
                company_id,
                official_label,
                street_name,
                street_number,
                postcode,
                city,
                coordinates
            ) VALUES ($1, $2, $3, $4, $5, $6, ST_SetSRID(ST_MakePoint($7, $8), 4326))
            ON CONFLICT (official_label) DO UPDATE SET
                updated_at = NOW()
            RETURNING 
                id,
                official_label,
                street_name,
                street_number,
                postcode,
                city,
                ST_Y(coordinates) as latitude,
                ST_X(coordinates) as longitude,
                door_code,
                has_mailbox_access,
                driver_notes
        "#;

        let row = sqlx::query(query)
            .bind(company_id)
            .bind(&formatted_address)
            .bind(&street_name)
            .bind(&street_number)
            .bind(&postcode)
            .bind(&city)
            .bind(longitude)
            .bind(latitude)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(CachedAddress {
                id: row.get("id"),
                official_label: row.get("official_label"),
                street_name: row.get("street_name"),
                street_number: row.get("street_number"),
                postcode: row.get("postcode"),
                city: row.get("city"),
                latitude: row.get("latitude"),
                longitude: row.get("longitude"),
                door_code: row.get("door_code"),
                has_mailbox_access: row.get("has_mailbox_access"),
                driver_notes: row.get("driver_notes"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Extraer número y nombre de calle de una dirección
    fn extract_street_components(&self, address: &str) -> (Option<String>, String) {
        // Regex para extraer número y calle: "123 Rue de la Paix" -> (Some("123"), "Rue de la Paix")
        let re = regex::Regex::new(r"^(\d+)\s+(.+)$").unwrap();
        
        if let Some(captures) = re.captures(address) {
            let number = captures.get(1).map(|m| m.as_str().to_string());
            let street = captures.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
            (number, street)
        } else {
            (None, address.to_string())
        }
    }

    /// Extraer código postal y ciudad de una dirección
    fn extract_postcode_city(&self, address: &str) -> (String, String) {
        // Regex para extraer código postal y ciudad: "75018 Paris" -> ("75018", "Paris")
        let re = regex::Regex::new(r"(\d{5})\s+([^,]+)").unwrap();
        
        if let Some(captures) = re.captures(address) {
            let postcode = captures.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let city = captures.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
            (postcode, city)
        } else {
            ("".to_string(), "".to_string())
        }
    }

    /// Limpiar caché de memoria
    pub fn clear_memory_cache(&mut self) {
        self.memory_cache.clear();
        log::info!("🧹 Caché de memoria limpiado");
    }

    /// Obtener estadísticas del caché
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.memory_cache.len(), 0) // (memoria, bd - se puede implementar después)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_street_components() {
        let service = AddressCacheService::new(
            // Mock pool para tests
            todo!(),
            GeocodingService::new("test".to_string()),
        );

        let (number, street) = service.extract_street_components("123 Rue de la Paix");
        assert_eq!(number, Some("123".to_string()));
        assert_eq!(street, "Rue de la Paix");

        let (number, street) = service.extract_street_components("Rue de la Paix");
        assert_eq!(number, None);
        assert_eq!(street, "Rue de la Paix");
    }

    #[test]
    fn test_extract_postcode_city() {
        let service = AddressCacheService::new(
            // Mock pool para tests
            todo!(),
            GeocodingService::new("test".to_string()),
        );

        let (postcode, city) = service.extract_postcode_city("123 Rue de la Paix, 75018 Paris");
        assert_eq!(postcode, "75018");
        assert_eq!(city, "Paris");
    }
}
