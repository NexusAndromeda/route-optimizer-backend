use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};

use crate::services::geocoding_service::GeocodingService;
use crate::services::address_cache_service::AddressCacheService;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct GeocodingApiRequest {
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct BatchGeocodingApiRequest {
    pub addresses: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct GeocodingApiResponse {
    pub success: bool,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub formatted_address: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchGeocodingApiResponse {
    pub success: bool,
    pub results: Vec<GeocodingApiResponse>,
    pub message: Option<String>,
    pub error: Option<String>,
}

pub fn create_geocoding_router() -> Router<AppState> {
    Router::new()
        .route("/geocoding", post(geocode_address))
        .route("/geocoding/batch", post(batch_geocode_addresses))
}

/// Endpoint para geocodificar una sola dirección
pub async fn geocode_address(
    State(state): State<AppState>,
    Json(request): Json<GeocodingApiRequest>,
) -> Result<Json<GeocodingApiResponse>, StatusCode> {
    log::info!("🗺️ Geocoding request received: {}", request.address);

    // Validar que la dirección no esté vacía
    if request.address.trim().is_empty() {
        log::warn!("⚠️ Empty address provided");
        return Ok(Json(GeocodingApiResponse {
            success: false,
            latitude: None,
            longitude: None,
            formatted_address: None,
            message: None,
            error: Some("Address cannot be empty".to_string()),
        }));
    }

    // Obtener el token de Mapbox del estado
    let mapbox_token = match &state.config.mapbox_token {
        Some(token) => token.clone(),
        None => {
            log::error!("❌ Mapbox token not configured");
            return Ok(Json(GeocodingApiResponse {
                success: false,
                latitude: None,
                longitude: None,
                formatted_address: None,
                message: None,
                error: Some("Mapbox token not configured".to_string()),
            }));
        }
    };

    // Crear el servicio de geocoding
    let geocoding_service = GeocodingService::new(mapbox_token);

    // Realizar la geocodificación
    match geocoding_service.geocode_address(&request.address).await {
        Ok(response) => {
            log::info!("✅ Geocoding successful for: {}", request.address);
            Ok(Json(GeocodingApiResponse {
                success: response.success,
                latitude: response.latitude,
                longitude: response.longitude,
                formatted_address: response.formatted_address,
                message: response.message,
                error: response.error,
            }))
        }
        Err(e) => {
            log::error!("❌ Geocoding error for {}: {}", request.address, e);
            Ok(Json(GeocodingApiResponse {
                success: false,
                latitude: None,
                longitude: None,
                formatted_address: None,
                message: None,
                error: Some(format!("Geocoding failed: {}", e)),
            }))
        }
    }
}

/// Endpoint para geocodificar múltiples direcciones en lote
pub async fn batch_geocode_addresses(
    State(state): State<AppState>,
    Json(request): Json<BatchGeocodingApiRequest>,
) -> Result<Json<BatchGeocodingApiResponse>, StatusCode> {
    log::info!("🗺️ Batch geocoding request received: {} addresses", request.addresses.len());

    // Validar que haya direcciones
    if request.addresses.is_empty() {
        log::warn!("⚠️ No addresses provided for batch geocoding");
        return Ok(Json(BatchGeocodingApiResponse {
            success: false,
            results: vec![],
            message: None,
            error: Some("No addresses provided".to_string()),
        }));
    }

    // Validar límite de direcciones (máximo 50 según la documentación)
    if request.addresses.len() > 50 {
        log::warn!("⚠️ Too many addresses for batch geocoding: {}", request.addresses.len());
        return Ok(Json(BatchGeocodingApiResponse {
            success: false,
            results: vec![],
            message: None,
            error: Some("Maximum 50 addresses allowed per batch".to_string()),
        }));
    }

    // Obtener el token de Mapbox del estado
    let mapbox_token = match &state.config.mapbox_token {
        Some(token) => token.clone(),
        None => {
            log::error!("❌ Mapbox token not configured");
            return Ok(Json(BatchGeocodingApiResponse {
                success: false,
                results: vec![],
                message: None,
                error: Some("Mapbox token not configured".to_string()),
            }));
        }
    };

    // Crear el servicio de geocoding
    let geocoding_service = GeocodingService::new(mapbox_token);

    // Realizar la geocodificación en lote
    match geocoding_service.batch_geocode(request.addresses).await {
        Ok(responses) => {
            let api_responses: Vec<GeocodingApiResponse> = responses
                .into_iter()
                .map(|response| GeocodingApiResponse {
                    success: response.success,
                    latitude: response.latitude,
                    longitude: response.longitude,
                    formatted_address: response.formatted_address,
                    message: response.message,
                    error: response.error,
                })
                .collect();

            log::info!("✅ Batch geocoding completed: {} results", api_responses.len());
            Ok(Json(BatchGeocodingApiResponse {
                success: true,
                results: api_responses,
                message: Some("Batch geocoding completed".to_string()),
                error: None,
            }))
        }
        Err(e) => {
            log::error!("❌ Batch geocoding error: {}", e);
            Ok(Json(BatchGeocodingApiResponse {
                success: false,
                results: vec![],
                message: None,
                error: Some(format!("Batch geocoding failed: {}", e)),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_geocoding_endpoint() {
        // Este test requiere configuración completa del estado
        // Se puede ejecutar con: cargo test -- --ignored
        println!("⚠️ Geocoding endpoint test requires full app state setup");
    }
}
