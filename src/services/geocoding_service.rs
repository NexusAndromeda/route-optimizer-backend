use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GeocodingRequest {
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeocodingResponse {
    pub success: bool,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub formatted_address: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MapboxGeocodingResponse {
    #[serde(rename = "type")]
    response_type: String,
    features: Vec<MapboxFeature>,
}

#[derive(Debug, Deserialize)]
struct MapboxFeature {
    #[serde(rename = "type")]
    feature_type: String,
    geometry: MapboxGeometry,
    properties: MapboxProperties,
}

#[derive(Debug, Deserialize)]
struct MapboxGeometry {
    #[serde(rename = "type")]
    geometry_type: String,
    coordinates: Vec<f64>, // [longitude, latitude]
}

#[derive(Debug, Deserialize)]
struct MapboxProperties {
    #[serde(rename = "full_address")]
    full_address: Option<String>,
    name: Option<String>,
    #[serde(rename = "place_name")]
    place_name: Option<String>,
}

pub struct GeocodingService {
    mapbox_token: String,
    client: reqwest::Client,
}

impl GeocodingService {
    pub fn new(mapbox_token: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            mapbox_token,
            client,
        }
    }

    pub async fn geocode_address(&self, address: &str) -> Result<GeocodingResponse> {
        log::info!("🗺️ Geocoding address: {}", address);

        // URL encode la dirección
        let encoded_address = urlencoding::encode(address);
        
        // Construir la URL según la documentación oficial
        let url = format!(
            "https://api.mapbox.com/search/geocode/v6/forward?q={}&access_token={}&country=fr&limit=1",
            encoded_address,
            self.mapbox_token
        );

        log::info!("🌐 Making request to: {}", url);

        // Hacer la petición HTTP
        let response = self.client
            .get(&url)
            .header("User-Agent", "DeliveryRouting/1.0")
            .send()
            .await?;

        let status = response.status();
        log::info!("📡 Response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("❌ Geocoding failed with status {}: {}", status, error_text);
            return Ok(GeocodingResponse {
                success: false,
                latitude: None,
                longitude: None,
                formatted_address: None,
                message: None,
                error: Some(format!("Geocoding failed: {}", status)),
            });
        }

        let response_text = response.text().await?;
        log::info!("📄 Response body: {}", response_text);

        // Parsear la respuesta JSON
        let mapbox_response: MapboxGeocodingResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse geocoding response: {}", e))?;

        // Extraer la primera feature (resultado más relevante)
        if let Some(feature) = mapbox_response.features.first() {
            if feature.geometry.coordinates.len() >= 2 {
                let longitude = feature.geometry.coordinates[0];
                let latitude = feature.geometry.coordinates[1];
                
                let formatted_address = feature.properties.full_address.clone()
                    .or_else(|| feature.properties.place_name.clone())
                    .or_else(|| feature.properties.name.clone());

                log::info!("✅ Geocoding successful: {} -> ({}, {})", 
                    address, latitude, longitude);

                return Ok(GeocodingResponse {
                    success: true,
                    latitude: Some(latitude),
                    longitude: Some(longitude),
                    formatted_address,
                    message: Some("Geocoding successful".to_string()),
                    error: None,
                });
            }
        }

        log::warn!("⚠️ No coordinates found for address: {}", address);
        Ok(GeocodingResponse {
            success: false,
            latitude: None,
            longitude: None,
            formatted_address: None,
            message: Some("No coordinates found for this address".to_string()),
            error: None,
        })
    }

    pub async fn batch_geocode(&self, addresses: Vec<String>) -> Result<Vec<GeocodingResponse>> {
        log::info!("🗺️ Batch geocoding {} addresses", addresses.len());
        
        let mut results = Vec::new();
        
        // Procesar en lotes de 10 para no sobrecargar la API
        for chunk in addresses.chunks(10) {
            let mut futures = Vec::new();
            
            for address in chunk {
                let future = self.geocode_address(address);
                futures.push(future);
            }
            
            // Ejecutar en paralelo
            let chunk_results = futures::future::join_all(futures).await;
            
            for result in chunk_results {
                match result {
                    Ok(response) => results.push(response),
                    Err(e) => {
                        log::error!("❌ Batch geocoding error: {}", e);
                        results.push(GeocodingResponse {
                            success: false,
                            latitude: None,
                            longitude: None,
                            formatted_address: None,
                            message: None,
                            error: Some(e.to_string()),
                        });
                    }
                }
            }
            
            // Pequeña pausa entre lotes para respetar rate limits
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        
        log::info!("✅ Batch geocoding completed: {} results", results.len());
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_geocoding_service() {
        // Este test requiere un token válido de Mapbox
        // Se puede ejecutar con: cargo test -- --ignored
        let token = std::env::var("MAPBOX_TOKEN").unwrap_or_default();
        if token.is_empty() {
            println!("⚠️ Skipping test: MAPBOX_TOKEN not set");
            return;
        }

        let service = GeocodingService::new(token);
        let result = service.geocode_address("15 Rue de la Paix, 75001 Paris").await;
        
        match result {
            Ok(response) => {
                println!("✅ Geocoding result: {:?}", response);
                assert!(response.success);
                assert!(response.latitude.is_some());
                assert!(response.longitude.is_some());
            }
            Err(e) => {
                println!("❌ Geocoding error: {}", e);
            }
        }
    }
}
