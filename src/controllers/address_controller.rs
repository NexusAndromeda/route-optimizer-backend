use crate::dto::address_dto::{SaveAddressRequest, AddressResponse, SearchAddressRequest};
use crate::dto::company_dto::ApiResponse;
use crate::repositories::address_repository::AddressRepository;
use crate::services::geocoding_service::GeocodingService;
use crate::utils::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;

pub struct AddressController {
    repository: AddressRepository,
}

impl AddressController {
    pub fn new(pool: PgPool) -> Self {
        Self {
            repository: AddressRepository::new(pool),
        }
    }

    pub async fn save(
        &self,
        request: SaveAddressRequest,
    ) -> Result<ApiResponse<AddressResponse>, AppError> {
        // Validar campos
        if request.address.trim().is_empty() {
            return Err(AppError::ValidationError("La dirección es requerida".to_string()));
        }

        // Crear dirección
        let address = self.repository.create(
            request.route_id,
            request.address,
            request.postal_code,
            request.door_codes,
            request.mailbox_access,
            request.access_instructions,
            request.latitude,
            request.longitude,
        ).await?;

        // Convertir a DTO
        let response = AddressResponse {
            id: address.id,
            route_id: address.route_id,
            address: address.address,
            postal_code: address.postal_code,
            door_codes: address.door_codes,
            mailbox_access: address.mailbox_access.unwrap_or(false),
            access_instructions: address.access_instructions,
            latitude: None, // TODO: Extraer de coordinates
            longitude: None,
            created_at: address.created_at,
        };

        Ok(ApiResponse::success_with_message(
            response,
            "Dirección guardada exitosamente".to_string()
        ))
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<AddressResponse, AppError> {
        let address = self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Dirección no encontrada".to_string()))?;

        Ok(AddressResponse {
            id: address.id,
            route_id: address.route_id,
            address: address.address,
            postal_code: address.postal_code,
            door_codes: address.door_codes,
            mailbox_access: address.mailbox_access.unwrap_or(false),
            access_instructions: address.access_instructions,
            latitude: None,
            longitude: None,
            created_at: address.created_at,
        })
    }

    pub async fn list_by_route(&self, route_id: Uuid) -> Result<Vec<AddressResponse>, AppError> {
        let addresses = self.repository.find_by_route(route_id).await?;

        let response = addresses.into_iter().map(|a| AddressResponse {
            id: a.id,
            route_id: a.route_id,
            address: a.address,
            postal_code: a.postal_code,
            door_codes: a.door_codes,
            mailbox_access: a.mailbox_access.unwrap_or(false),
            access_instructions: a.access_instructions,
            latitude: None,
            longitude: None,
            created_at: a.created_at,
        }).collect();

        Ok(response)
    }

    pub async fn search(&self, request: SearchAddressRequest) -> Result<Vec<AddressResponse>, AppError> {
        if let Some(address_pattern) = request.address {
            let addresses = self.repository.search_by_address(&address_pattern).await?;

            let response = addresses.into_iter().map(|a| AddressResponse {
                id: a.id,
                route_id: a.route_id,
                address: a.address,
                postal_code: a.postal_code,
                door_codes: a.door_codes,
                mailbox_access: a.mailbox_access.unwrap_or(false),
                access_instructions: a.access_instructions,
                latitude: None,
                longitude: None,
                created_at: a.created_at,
            }).collect();

            Ok(response)
        } else {
            Ok(vec![])
        }
    }

    pub async fn update_details(
        &self,
        id: Uuid,
        door_codes: Option<String>,
        mailbox_access: Option<bool>,
        access_instructions: Option<String>,
    ) -> Result<ApiResponse<AddressResponse>, AppError> {
        let address = self.repository.update(id, door_codes, mailbox_access, access_instructions).await?;

        let response = AddressResponse {
            id: address.id,
            route_id: address.route_id,
            address: address.address,
            postal_code: address.postal_code,
            door_codes: address.door_codes,
            mailbox_access: address.mailbox_access.unwrap_or(false),
            access_instructions: address.access_instructions,
            latitude: None,
            longitude: None,
            created_at: address.created_at,
        };

        Ok(ApiResponse::success_with_message(
            response,
            "Dirección actualizada exitosamente".to_string()
        ))
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id).await?;
        Ok(())
    }

    /// Geocodificar una dirección usando Mapbox
    pub async fn geocode_address(&self, address: String) -> Result<ApiResponse<serde_json::Value>, AppError> {
        if address.trim().is_empty() {
            return Err(AppError::ValidationError("La dirección es requerida".to_string()));
        }

        log::info!("🌍 Geocodificando dirección: {}", address);

        // Crear servicio de geocodificación (necesita token de Mapbox)
        let mapbox_token = std::env::var("MAPBOX_ACCESS_TOKEN")
            .map_err(|_| AppError::Internal("MAPBOX_ACCESS_TOKEN no configurado".to_string()))?;
        let geocoding_service = GeocodingService::new(mapbox_token);
        
        // Geocodificar
        match geocoding_service.geocode_address(&address).await {
            Ok(response) => {
                if response.success {
                    log::info!("✅ Geocodificación exitosa: {} -> ({}, {})", 
                        address, 
                        response.latitude.unwrap_or(0.0), 
                        response.longitude.unwrap_or(0.0)
                    );
                    
                    let result = serde_json::json!({
                        "success": true,
                        "latitude": response.latitude,
                        "longitude": response.longitude,
                        "formatted_address": response.formatted_address,
                        "message": "Dirección geocodificada exitosamente"
                    });
                    
                    Ok(ApiResponse::success_with_message(result, "Geocodificación exitosa".to_string()))
                } else {
                    let error_msg = response.message.clone().unwrap_or_default();
                    log::warn!("⚠️ Geocodificación falló: {}", error_msg);
                    Err(AppError::ValidationError(error_msg))
                }
            }
            Err(e) => {
                log::error!("❌ Error en geocodificación: {}", e);
                Err(AppError::Internal(format!("Error en geocodificación: {}", e)))
            }
        }
    }
}
