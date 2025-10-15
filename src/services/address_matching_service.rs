use crate::models::address::{Address, AddressSearch, ColisPriveAddress};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::{info, warn};
use sqlx::PgPool;

/// Servicio para hacer matching de direcciones con la base de datos oficial
/// Utiliza un cache en memoria para búsquedas rápidas
pub struct AddressMatchingService {
    pool: Arc<PgPool>,
    // Cache de direcciones: "RUE HERMEL 75018" -> Address
    address_cache: Arc<RwLock<HashMap<String, Address>>>,
}

impl AddressMatchingService {
    /// Crea una nueva instancia del servicio y carga todas las direcciones en cache
    pub async fn new(pool: Arc<PgPool>) -> Result<Self> {
        let service = Self {
            pool: pool.clone(),
            address_cache: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Cargar todas las direcciones al iniciar
        service.load_addresses_into_cache().await?;
        
        Ok(service)
    }
    
    /// Carga todas las direcciones de la BD en el cache
    async fn load_addresses_into_cache(&self) -> Result<()> {
        info!("🔄 Cargando direcciones en cache...");
        
        let addresses = self.get_all_addresses().await?;
        let mut cache = self.address_cache.write().await;
        
        for address in addresses {
            let search_key = address.search_key();
            cache.insert(search_key, address);
        }
        
        info!("✅ {} direcciones cargadas en cache", cache.len());
        Ok(())
    }
    
    /// Obtiene todas las direcciones de la BD
    async fn get_all_addresses(&self) -> Result<Vec<Address>> {
        // Por ahora retorna vacío, se implementará cuando la tabla exista
        Ok(vec![])
    }
    
    /// Busca una dirección en el cache
    pub async fn find_address(&self, street_name: &str, postcode: &str) -> Option<Address> {
        let search_key = format!("{} {}", street_name, postcode);
        let cache = self.address_cache.read().await;
        cache.get(&search_key).cloned()
    }
    
    /// Busca una dirección usando AddressSearch
    pub async fn find_address_by_search(&self, search: &AddressSearch) -> Option<Address> {
        self.find_address(&search.street_name, &search.postcode).await
    }
    
    /// Busca una dirección de Colis Privé en la BD oficial
    pub async fn find_colis_prive_address(&self, colis_addr: &ColisPriveAddress) -> Option<Address> {
        self.find_address(&colis_addr.libelle_voie, &colis_addr.code_postal).await
    }
    
    /// Crea una nueva dirección si no existe (para geocodificación on-the-go)
    pub async fn create_address_if_not_exists(
        &self, 
        street_name: String, 
        postcode: String,
        city: String,
        latitude: f64,
        longitude: f64,
        company_id: Option<uuid::Uuid>
    ) -> Result<Address> {
        // Verificar si ya existe
        if let Some(existing) = self.find_address(&street_name, &postcode).await {
            return Ok(existing);
        }
        
        // Crear nueva dirección
        let official_label = format!("{} {} {}", 
            street_name, postcode, city);
            
        let new_address = self.create_address(
            company_id,
            official_label,
            street_name,
            None, // street_number
            postcode,
            city,
            latitude,
            longitude,
            None, // door_code
            false, // has_mailbox_access
            None, // driver_notes
            None, // last_updated_by
        ).await?;
        
        // Agregar al cache
        let search_key = new_address.search_key();
        let mut cache = self.address_cache.write().await;
        cache.insert(search_key, new_address.clone());
        
        info!("✅ Nueva dirección creada: {}", new_address.official_label);
        Ok(new_address)
    }
    
    /// Actualiza los datos del chofer para una dirección
    pub async fn update_driver_data(
        &self,
        address_id: uuid::Uuid,
        door_code: Option<String>,
        has_mailbox_access: bool,
        driver_notes: Option<String>,
        updated_by: String,
    ) -> Result<Address> {
        // Actualizar en BD
        let updated_address = self.update_address_driver_data_in_db(
            address_id,
            door_code.clone(),
            has_mailbox_access,
            driver_notes.clone(),
            updated_by.clone(),
        ).await?;
        
        // Actualizar cache
        let search_key = updated_address.search_key();
        let mut cache = self.address_cache.write().await;
        cache.insert(search_key, updated_address.clone());
        
        info!("✅ Datos del chofer actualizados para: {}", updated_address.official_label);
        Ok(updated_address)
    }
    
    /// Obtiene estadísticas del cache
    pub async fn get_cache_stats(&self) -> Result<(usize, Vec<String>)> {
        let cache = self.address_cache.read().await;
        let count = cache.len();
        let sample_keys: Vec<String> = cache.keys().take(5).cloned().collect();
        Ok((count, sample_keys))
    }
    
    /// Crea una dirección en la BD
    async fn create_address(
        &self,
        company_id: Option<uuid::Uuid>,
        official_label: String,
        street_name: String,
        street_number: Option<String>,
        postcode: String,
        city: String,
        latitude: f64,
        longitude: f64,
        door_code: Option<String>,
        has_mailbox_access: bool,
        driver_notes: Option<String>,
        last_updated_by: Option<String>,
    ) -> Result<Address> {
        // Por ahora retornamos una dirección mock
        // Se implementará cuando la tabla exista
        Ok(Address {
            id: uuid::Uuid::new_v4(),
            company_id,
            official_label,
            street_name,
            street_number,
            postcode,
            city,
            latitude,
            longitude,
            door_code,
            has_mailbox_access,
            driver_notes,
            last_updated_by,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    
    /// Actualiza datos del chofer en la BD
    async fn update_address_driver_data_in_db(
        &self,
        address_id: uuid::Uuid,
        door_code: Option<String>,
        has_mailbox_access: bool,
        driver_notes: Option<String>,
        updated_by: String,
    ) -> Result<Address> {
        // Por ahora retornamos una dirección mock
        // Se implementará cuando la tabla exista
        Ok(Address {
            id: address_id,
            company_id: None,
            official_label: "Mock Address".to_string(),
            street_name: "Mock Street".to_string(),
            street_number: None,
            postcode: "75000".to_string(),
            city: "Paris".to_string(),
            latitude: 0.0,
            longitude: 0.0,
            door_code,
            has_mailbox_access,
            driver_notes,
            last_updated_by: Some(updated_by),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
}
