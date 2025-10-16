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

    /// Optimizar una ruta usando Mapbox Optimization API v2 (Beta)
    pub async fn optimize_route(
        &self,
        packages: Vec<OptimizationPackage>,
        warehouse_location: Option<(f64, f64)>, // (longitude, latitude)
    ) -> Result<OptimizationResponse> {
        log::info!("🚀 Iniciando optimización con Mapbox v2 para {} paquetes", packages.len());

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

        // API v2 soporta hasta 1000 locations
        if packages_with_coords.len() > 1000 {
            log::warn!("⚠️ API v2 limita a 1000 locations, usando solo las primeras 1000");
        }

        let packages_to_optimize: Vec<_> = packages_with_coords.iter().take(1000).cloned().collect();
        log::info!("📍 Optimizando {} paquetes con coordenadas válidas", packages_to_optimize.len());

        // Construir routing problem document para v2
        let routing_problem = self.build_routing_problem_v2(&packages_to_optimize, warehouse_location)?;

        log::info!("📋 Enviando routing problem a Mapbox Optimization API v2");

        // Paso 1: Enviar el problema (POST)
        let submit_response = self.submit_routing_problem_v2(&routing_problem).await?;
        let job_id = submit_response.id;
        
        log::info!("📤 Routing problem enviado, job_id: {}", job_id);

        // Paso 2: Polling para obtener la solución (GET)
        let solution = self.poll_solution_v2(&job_id).await?;
        
        log::info!("🎯 Solución obtenida de Mapbox v2");

        // Paso 3: Procesar la solución y convertir a nuestro formato
        let optimized_packages = self.process_solution_v2(&solution, &packages_to_optimize)?;

        log::info!("✅ Optimización completada: {} paquetes optimizados", optimized_packages.len());

        Ok(OptimizationResponse {
            success: true,
            message: Some("Ruta optimizada exitosamente con Mapbox v2".to_string()),
            data: Some(OptimizationData {
                matricule_chauffeur: None,
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

    /// Construir routing problem document para v2
    fn build_routing_problem_v2(
        &self,
        packages: &[OptimizationPackage],
        warehouse_location: Option<(f64, f64)>,
    ) -> Result<MapboxOptimizationRequest> {
        let mut locations = Vec::new();
        let mut services = Vec::new();

        // Agregar warehouse como location si existe
        if let Some((lon, lat)) = warehouse_location {
            locations.push(MapboxLocation {
                name: "warehouse".to_string(),
                coordinates: [lon, lat],
            });
        } else if let Some(first_pkg) = packages.first() {
            // Si no hay warehouse, usar el primer paquete como inicio
            locations.push(MapboxLocation {
                name: "start".to_string(),
                coordinates: [
                    first_pkg.coord_x_destinataire.unwrap(),
                    first_pkg.coord_y_destinataire.unwrap(),
                ],
            });
        }

        // Agregar cada paquete como location y service
        for (idx, pkg) in packages.iter().enumerate() {
            let location_name = format!("delivery-{}", idx);
            
            locations.push(MapboxLocation {
                name: location_name.clone(),
                coordinates: [
                    pkg.coord_x_destinataire.unwrap(),
                    pkg.coord_y_destinataire.unwrap(),
                ],
            });

            services.push(MapboxService {
                name: format!("service-{}", idx),
                location: location_name,
                duration: 120, // 2 minutos por entrega (ajustable)
                size: None,
            });
        }

        // Crear vehículo
        let start_location = if warehouse_location.is_some() {
            "warehouse".to_string()
        } else {
            "start".to_string()
        };

        let vehicles = vec![MapboxVehicle {
            name: "vehicle-1".to_string(),
            start_location: start_location.clone(),
            end_location: start_location, // Round trip
            capacity: None,
        }];

        // Opciones de optimización
        let options = Some(MapboxOptions {
            objectives: Some(vec!["min-schedule-completion-time".to_string()]),
        });

        Ok(MapboxOptimizationRequest {
            version: 1,
            locations,
            vehicles,
            services,
            options,
        })
    }

    /// Enviar routing problem a Mapbox v2 (POST)
    async fn submit_routing_problem_v2(
        &self,
        routing_problem: &MapboxOptimizationRequest,
    ) -> Result<MapboxSubmitResponse> {
        let url = format!(
            "https://api.mapbox.com/optimized-trips/v2?access_token={}",
            self.mapbox_token
        );

        log::info!("📤 POST a: {}", url);

        let response = self.client
            .post(&url)
            .json(routing_problem)
            .header("Content-Type", "application/json")
            .header("User-Agent", "RouteOptimizer/1.0")
            .send()
            .await?;

        let status = response.status();
        
        if status.as_u16() != 202 {
            let error_text = response.text().await?;
            return Err(anyhow!("Mapbox v2 submit error {}: {}", status, error_text));
        }

        let submit_response: MapboxSubmitResponse = response.json().await?;
        log::info!("✅ Submitted successfully, status: {}", submit_response.status);

        Ok(submit_response)
    }

    /// Polling para obtener la solución v2 (GET)
    async fn poll_solution_v2(&self, job_id: &str) -> Result<MapboxOptimizationV2Response> {
        let url = format!(
            "https://api.mapbox.com/optimized-trips/v2/{}?access_token={}",
            job_id, self.mapbox_token
        );

        let max_attempts = 30; // 30 intentos
        let poll_interval = std::time::Duration::from_secs(2); // 2 segundos entre intentos

        for attempt in 1..=max_attempts {
            log::info!("🔍 Polling intento {}/{}: {}", attempt, max_attempts, url);

            let response = self.client
                .get(&url)
                .header("User-Agent", "RouteOptimizer/1.0")
                .send()
                .await?;

            let status = response.status();

            if status.as_u16() == 202 {
                // Aún procesando
                log::info!("⏳ Solución aún procesando, esperando {} segundos...", poll_interval.as_secs());
                tokio::time::sleep(poll_interval).await;
                continue;
            }

            if status.as_u16() == 200 {
                // Solución lista
                let solution: MapboxOptimizationV2Response = response.json().await?;
                log::info!("✅ Solución lista después de {} intentos", attempt);
                return Ok(solution);
            }

            // Otro error
            let error_text = response.text().await?;
            return Err(anyhow!("Mapbox v2 poll error {}: {}", status, error_text));
        }

        Err(anyhow!("Timeout esperando solución después de {} intentos", max_attempts))
    }

    /// Procesar solución v2 y convertir a nuestro formato
    fn process_solution_v2(
        &self,
        solution: &MapboxOptimizationV2Response,
        packages: &[OptimizationPackage],
    ) -> Result<Vec<OptimizedPackage>> {
        let mut optimized_packages = Vec::new();

        // Verificar si hay servicios dropped
        if let Some(dropped) = &solution.dropped {
            if let Some(services) = &dropped.services {
                if !services.is_empty() {
                    log::warn!("⚠️ {} servicios no pudieron ser incluidos en la solución", services.len());
                }
            }
        }

        // Obtener la primera ruta (asumimos un solo vehículo)
        let route = solution.routes.first()
            .ok_or_else(|| anyhow!("No hay rutas en la solución"))?;

        log::info!("📍 Procesando ruta con {} stops", route.stops.len());

        // Procesar cada stop de tipo "service"
        let mut order = 1;
        for stop in &route.stops {
            if stop.stop_type == "service" {
                if let Some(service_names) = &stop.services {
                    for service_name in service_names {
                        // Extraer el índice del nombre del servicio (ej: "service-0" → 0)
                        if let Some(idx_str) = service_name.strip_prefix("service-") {
                            if let Ok(pkg_idx) = idx_str.parse::<usize>() {
                                if let Some(pkg) = packages.get(pkg_idx) {
                                    let mut optimized_pkg = OptimizedPackage::from(pkg.clone());
                                    optimized_pkg.numero_ordre = Some(order);
                                    optimized_pkg.num_ordre_passage_prevu = Some(order);
                                    optimized_pkg.eta = Some(stop.eta.clone());
                                    
                                    optimized_packages.push(optimized_pkg);
                                    order += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        if optimized_packages.is_empty() {
            return Err(anyhow!("No se pudieron extraer paquetes optimizados de la solución"));
        }

        log::info!("✅ {} paquetes procesados de la solución v2", optimized_packages.len());
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
