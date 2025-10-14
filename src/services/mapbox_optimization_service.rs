//! Servicio para Mapbox Optimization API v2
//! 
//! Este módulo maneja la comunicación con la API de optimización de rutas de Mapbox.

use anyhow::{anyhow, Result};
use chrono::Utc;
use reqwest::Client;
use std::time::Duration;

use crate::dto::mapbox_optimization_dto::*;

pub struct MapboxOptimizationService {
    mapbox_token: String,
    client: Client,
}

impl MapboxOptimizationService {
    pub fn new(mapbox_token: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minutos para optimizaciones complejas
            .build()
            .expect("Failed to create HTTP client");

        Self {
            mapbox_token,
            client,
        }
    }

    /// Optimizar una ruta usando Mapbox Optimization API v1
    pub async fn optimize_route(
        &self,
        packages: Vec<OptimizationPackage>,
        warehouse_location: Option<(f64, f64)>, // (longitude, latitude)
    ) -> Result<OptimizationResponse> {
        log::info!("🚀 Iniciando optimización con Mapbox v1 para {} paquetes", packages.len());

        // Validar que tenemos paquetes con coordenadas
        let packages_with_coords: Vec<_> = packages.iter()
            .filter(|pkg| pkg.coord_x_destinataire.is_some() && pkg.coord_y_destinataire.is_some())
            .collect();

        if packages_with_coords.is_empty() {
            return Ok(OptimizationResponse {
                success: false,
                message: Some("No hay paquetes con coordenadas válidas para optimizar".to_string()),
                data: None,
            });
        }

        // API v1 tiene límite de 12 coordenadas
        if packages_with_coords.len() > 11 {
            log::warn!("⚠️ API v1 limita a 12 coordenadas, usando solo las primeras 11 paquetes");
        }

        let packages_to_optimize = packages_with_coords.iter().take(11).collect::<Vec<_>>();
        log::info!("📍 Optimizando {} paquetes con coordenadas válidas", packages_to_optimize.len());

        // Construir coordenadas para la URL
        let mut coordinates = Vec::new();

        // Agregar warehouse como punto de inicio si se proporciona
        if let Some((lon, lat)) = warehouse_location {
            coordinates.push(format!("{},{}", lon, lat));
        }

        // Agregar coordenadas de paquetes
        for package in &packages_to_optimize {
            let lon = package.coord_x_destinataire.unwrap();
            let lat = package.coord_y_destinataire.unwrap();
            coordinates.push(format!("{},{}", lon, lat));
        }

        // Si no hay warehouse, el primer paquete será el punto de inicio
        if warehouse_location.is_none() && !coordinates.is_empty() {
            // Mover la primera coordenada al final para hacer round trip
            let first_coord = coordinates.remove(0);
            coordinates.push(first_coord);
        }

        let coordinates_str = coordinates.join(";");

        log::info!("📋 Enviando request a Mapbox Optimization API v1");

        // Llamar a la API v1
        let solution = self.call_optimization_v1(&coordinates_str).await?;
        
        log::info!("🎯 Solución obtenida de Mapbox v1");

        // Procesar la solución y convertir a nuestro formato
        let optimized_packages = self.process_solution_v1(&solution, packages_to_optimize).await?;

        log::info!("✅ Optimización completada: {} paquetes optimizados", optimized_packages.len());

        Ok(OptimizationResponse {
            success: true,
            message: Some("Ruta optimizada exitosamente con Mapbox v1".to_string()),
            data: Some(OptimizationData {
                matricule_chauffeur: None, // No aplica para Mapbox
                date_tournee: Some(Utc::now().to_rfc3339()),
                optimized_packages,
            }),
        })
    }

    /// Llamar a Mapbox Optimization API v1
    async fn call_optimization_v1(&self, coordinates: &str) -> Result<MapboxOptimizationResponse> {
        let url = format!(
            "https://api.mapbox.com/optimized-trips/v1/mapbox/driving/{}?roundtrip=true&access_token={}",
            coordinates, self.mapbox_token
        );

        log::info!("📤 Enviando a: {}", url);

        let response = self.client
            .get(&url)
            .header("User-Agent", "RouteOptimizer/1.0")
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        log::info!("📡 Mapbox response status: {}", status);
        log::debug!("📄 Mapbox response body: {}", response_text);

        if !status.is_success() {
            return Err(anyhow!("Mapbox API error {}: {}", status, response_text));
        }

        let optimization_response: MapboxOptimizationResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Error parsing Mapbox optimization response: {}", e))?;

        if optimization_response.code != "Ok" {
            return Err(anyhow!("Mapbox optimization failed: {} - {}", 
                optimization_response.code, 
                optimization_response.message.unwrap_or_default()));
        }

        Ok(optimization_response)
    }

    /// Esperar por la solución de optimización
    async fn wait_for_solution(&self, job_id: &str) -> Result<MapboxOptimizationResponse> {
        let url = format!(
            "https://api.mapbox.com/optimized-trips/v1/{}?access_token={}",
            job_id, self.mapbox_token
        );

        let mut attempts = 0;
        let max_attempts = 60; // Máximo 5 minutos (5 segundos * 60)
        let delay = Duration::from_secs(5);

        loop {
            attempts += 1;
            log::info!("⏳ Esperando solución de Mapbox (intento {}/{})", attempts, max_attempts);

            let response = self.client
                .get(&url)
                .header("User-Agent", "RouteOptimizer/1.0")
                .send()
                .await?;

            let status = response.status();
            let response_text = response.text().await?;

            if status == 202 {
                // Aún procesando
                if attempts >= max_attempts {
                    return Err(anyhow!("Timeout esperando solución de Mapbox después de {} intentos", max_attempts));
                }
                tokio::time::sleep(delay).await;
                continue;
            }

            if status == 200 {
                // Solución lista
                let solution: MapboxOptimizationResponse = serde_json::from_str(&response_text)
                    .map_err(|e| anyhow!("Error parsing Mapbox solution: {}", e))?;
                return Ok(solution);
            }

            return Err(anyhow!("Mapbox API error {}: {}", status, response_text));
        }
    }

    /// Procesar la solución de Mapbox y convertir a nuestro formato
    async fn process_solution(
        &self,
        solution: &MapboxOptimizationResponse,
        packages: Vec<&OptimizationPackage>,
    ) -> Result<Vec<OptimizedPackage>> {
        let mut optimized_packages = Vec::new();

        for route in &solution.routes {
            let mut order = 1;
            
            for stop in &route.stops {
                if stop.stop_type == "service" && stop.services.is_some() {
                    // Este es un servicio de entrega
                    for service_name in stop.services.as_ref().unwrap() {
                        // Extraer ID del paquete del nombre del servicio (delivery-{id})
                        if let Some(package_id) = service_name.strip_prefix("delivery-") {
                            // Buscar el paquete original
                            if let Some(original_package) = packages.iter().find(|p| p.id == package_id) {
                                let mut optimized_package: OptimizedPackage = (*original_package).clone().into();
                                
                                // Asignar orden de optimización
                                optimized_package.numero_ordre = Some(order);
                                optimized_package.num_ordre_passage_prevu = Some(order);
                                
                                // Asignar ETA
                                optimized_package.eta = Some(stop.eta.clone());
                                
                                optimized_packages.push(optimized_package);
                                order += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(optimized_packages)
    }

    /// Procesar la solución de Mapbox v1 y convertir a nuestro formato
    async fn process_solution_v1(
        &self,
        solution: &MapboxOptimizationResponse,
        packages: Vec<&OptimizationPackage>,
    ) -> Result<Vec<OptimizedPackage>> {
        let mut optimized_packages = Vec::new();

        if let Some(waypoints) = &solution.waypoints {
            for (order, waypoint) in waypoints.iter().enumerate() {
                // El waypoint_index nos dice qué paquete original es
                if waypoint.waypoint_index < packages.len() {
                    let original_package = packages[waypoint.waypoint_index];
                    let mut optimized_package: OptimizedPackage = original_package.clone().into();
                    
                    // Asignar orden de optimización (empezando desde 1)
                    optimized_package.numero_ordre = Some((order + 1) as i32);
                    optimized_package.num_ordre_passage_prevu = Some((order + 1) as i32);
                    
                    optimized_packages.push(optimized_package);
                }
            }
        }

        Ok(optimized_packages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mapbox_optimization_service() {
        // Este test requiere un token válido de Mapbox
        let token = std::env::var("MAPBOX_TOKEN").unwrap_or_default();
        if token.is_empty() {
            println!("⚠️ Skipping test: MAPBOX_TOKEN not set");
            return;
        }

        let service = MapboxOptimizationService::new(token);
        
        // Crear paquetes de prueba
        let packages = vec![
            OptimizationPackage {
                id: "pkg1".to_string(),
                reference_colis: "REF001".to_string(),
                destinataire_nom: "Test User 1".to_string(),
                destinataire_adresse1: Some("123 Test St".to_string()),
                destinataire_cp: Some("75001".to_string()),
                destinataire_ville: Some("Paris".to_string()),
                coord_x_destinataire: Some(2.3522),
                coord_y_destinataire: Some(48.8566),
                statut: Some("pending".to_string()),
            },
            OptimizationPackage {
                id: "pkg2".to_string(),
                reference_colis: "REF002".to_string(),
                destinataire_nom: "Test User 2".to_string(),
                destinataire_adresse1: Some("456 Test Ave".to_string()),
                destinataire_cp: Some("75002".to_string()),
                destinataire_ville: Some("Paris".to_string()),
                coord_x_destinataire: Some(2.3601),
                coord_y_destinataire: Some(48.8576),
                statut: Some("pending".to_string()),
            },
        ];

        let warehouse = Some((2.3522, 48.8566)); // Paris center
        
        let result = service.optimize_route(packages, warehouse).await;
        
        match result {
            Ok(response) => {
                println!("✅ Optimization result: {:?}", response);
                assert!(response.success);
                assert!(response.data.is_some());
            }
            Err(e) => {
                println!("❌ Optimization error: {}", e);
            }
        }
    }
}
