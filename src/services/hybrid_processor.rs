//! Procesador híbrido para combinar datos básicos con API detalle
//! 
//! Implementa la estrategia híbrida que combina eficientemente los datos
//! básicos de Colis Privé con información detallada del API detalle.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::clients::{ColisPriveWebClient, ColisDetailResponse};
use crate::cache::{DetailCache, CacheStrategy};
use crate::analysis::delivery_classifier::DeliveryType;
use crate::models::package::{Package, DeliveryStatus};
use crate::config::environment::EnvironmentConfig;

/// Procesador híbrido para optimización de rutas
pub struct HybridProcessor {
    /// Cliente para API detalle
    client: ColisPriveWebClient,
    /// Cache inteligente para datos detallados
    detail_cache: DetailCache,
    /// Estrategia de cache
    cache_strategy: CacheStrategy,
}

/// Resultado del procesamiento híbrido
#[derive(Debug, Serialize, Deserialize)]
pub struct HybridProcessingResult {
    /// Paquetes procesados exitosamente
    pub processed_packages: Vec<EnrichedPackage>,
    /// Paquetes que fallaron en el procesamiento
    pub failed_packages: Vec<FailedPackage>,
    /// Estadísticas del procesamiento
    pub stats: ProcessingStats,
}

/// Paquete enriquecido con datos híbridos
#[derive(Debug, Serialize, Deserialize)]
pub struct EnrichedPackage {
    /// Datos básicos del paquete
    pub basic_data: Package,
    /// Datos detallados del API detalle
    pub detail_data: Option<ColisDetailResponse>,
    /// Tipo de entrega clasificado
    pub delivery_type: DeliveryType,
    /// Calidad de geocoding
    pub geocoding_quality: String,
    /// Prioridad para llamada al API detalle
    pub detail_priority: DetailPriority,
    /// Datos enriquecidos combinados
    pub enriched_data: EnrichedData,
}

/// Datos enriquecidos combinados
#[derive(Debug, Serialize, Deserialize)]
pub struct EnrichedData {
    /// Dirección completa y validada
    pub complete_address: String,
    /// Coordenadas precisas
    pub coordinates: Option<(f64, f64)>,
    /// Código de barras completo
    pub complete_barcode: Option<String>,
    /// Datos físicos del paquete
    pub physical_data: Option<PhysicalData>,
    /// Información de contacto
    pub contact_info: Option<ContactInfo>,
    /// Instrucciones de entrega
    pub delivery_instructions: Option<String>,
    /// Historial de trazabilidad
    pub tracking_history: Option<Vec<TrackingEvent>>,
    /// Comentarios especiales
    pub special_comments: Option<String>,
}

/// Datos físicos del paquete
#[derive(Debug, Serialize, Deserialize)]
pub struct PhysicalData {
    pub weight: Option<f64>,
    pub weight_unit: Option<String>,
    pub dimensions: Option<Dimensions>,
    pub value: Option<f64>,
    pub currency: Option<String>,
}

/// Dimensiones del paquete
#[derive(Debug, Serialize, Deserialize)]
pub struct Dimensions {
    pub length: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub unit: Option<String>,
}

/// Información de contacto
#[derive(Debug, Serialize, Deserialize)]
pub struct ContactInfo {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

/// Evento de trazabilidad
#[derive(Debug, Serialize, Deserialize)]
pub struct TrackingEvent {
    pub date: String,
    pub time: String,
    pub status: String,
    pub description: Option<String>,
    pub location: Option<String>,
}

/// Paquete que falló en el procesamiento
#[derive(Debug, Serialize, Deserialize)]
pub struct FailedPackage {
    pub ref_colis: String,
    pub error: String,
    pub stage: ProcessingStage,
}

/// Etapa del procesamiento donde falló
#[derive(Debug, Serialize, Deserialize)]
pub enum ProcessingStage {
    BasicData,
    Classification,
    DetailApi,
    Enrichment,
}

/// Prioridad para llamada al API detalle
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum DetailPriority {
    /// Alta prioridad - paquete crítico
    High,
    /// Media prioridad - paquete importante
    Medium,
    /// Baja prioridad - paquete estándar
    Low,
    /// No necesita API detalle
    None,
}

/// Estadísticas del procesamiento
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub total_packages: usize,
    pub processed_successfully: usize,
    pub failed_packages: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub detail_api_calls: usize,
    pub processing_time_ms: u64,
}

impl HybridProcessor {
    /// Crear nuevo procesador híbrido
    pub fn new(cache_strategy: CacheStrategy, config: &EnvironmentConfig) -> Result<Self> {
        let client = ColisPriveWebClient::new(
            config.colis_prive_auth_url.clone(),
            config.colis_prive_tournee_url.clone(),
            config.colis_prive_detail_url.clone(),
        )?;
        let detail_cache = DetailCache::new(crate::clients::DetailCacheConfig::default());
        
        Ok(Self {
            client,
            detail_cache,
            cache_strategy,
        })
    }

    /// Procesar paquetes con estrategia híbrida
    pub async fn process_packages(
        &self,
        packages: Vec<Package>,
        sso_token: &str,
    ) -> Result<HybridProcessingResult> {
        let start_time = std::time::Instant::now();
        let mut stats = ProcessingStats {
            total_packages: packages.len(),
            processed_successfully: 0,
            failed_packages: 0,
            cache_hits: 0,
            cache_misses: 0,
            detail_api_calls: 0,
            processing_time_ms: 0,
        };

        let mut processed_packages = Vec::new();
        let mut failed_packages = Vec::new();

        // Fase 1: Clasificar paquetes y determinar prioridades
        let mut high_priority_packages = Vec::new();
        let mut medium_priority_packages = Vec::new();
        let mut low_priority_packages = Vec::new();

        for package in packages {
            match self.classify_and_prioritize_package(&package).await {
                Ok((delivery_type, priority)) => {
                    match priority {
                        DetailPriority::High => high_priority_packages.push((package, delivery_type)),
                        DetailPriority::Medium => medium_priority_packages.push((package, delivery_type)),
                        DetailPriority::Low => low_priority_packages.push((package, delivery_type)),
                        DetailPriority::None => {
                            // Procesar sin API detalle
                            match self.process_without_detail(package.clone(), delivery_type).await {
                                Ok(enriched) => {
                                    processed_packages.push(enriched);
                                    stats.processed_successfully += 1;
                                }
                                Err(e) => {
                                    failed_packages.push(FailedPackage {
                                        ref_colis: package.tracking_number.clone(),
                                        error: e.to_string(),
                                        stage: ProcessingStage::BasicData,
                                    });
                                    stats.failed_packages += 1;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    failed_packages.push(FailedPackage {
                        ref_colis: package.tracking_number.clone(),
                        error: e.to_string(),
                        stage: ProcessingStage::Classification,
                    });
                    stats.failed_packages += 1;
                }
            }
        }

        // Fase 2: Procesar paquetes de alta prioridad
        if !high_priority_packages.is_empty() {
            let ref_colis_list: Vec<String> = high_priority_packages
                .iter()
                .map(|(p, _)| p.tracking_number.clone())
                .collect();
            
            match self.process_priority_batch(high_priority_packages, &ref_colis_list, sso_token).await {
                Ok(mut results) => {
                    processed_packages.append(&mut results);
                    stats.processed_successfully += results.len();
                }
                Err(e) => {
                    warn!("Error procesando paquetes de alta prioridad: {}", e);
                }
            }
        }

        // Fase 3: Procesar paquetes de media prioridad
        if !medium_priority_packages.is_empty() {
            let ref_colis_list: Vec<String> = medium_priority_packages
                .iter()
                .map(|(p, _)| p.tracking_number.clone())
                .collect();
            
            match self.process_priority_batch(medium_priority_packages, &ref_colis_list, sso_token).await {
                Ok(mut results) => {
                    processed_packages.append(&mut results);
                    stats.processed_successfully += results.len();
                }
                Err(e) => {
                    warn!("Error procesando paquetes de media prioridad: {}", e);
                }
            }
        }

        // Fase 4: Procesar paquetes de baja prioridad
        if !low_priority_packages.is_empty() {
            let ref_colis_list: Vec<String> = low_priority_packages
                .iter()
                .map(|(p, _)| p.tracking_number.clone())
                .collect();
            
            match self.process_priority_batch(low_priority_packages, &ref_colis_list, sso_token).await {
                Ok(mut results) => {
                    processed_packages.append(&mut results);
                    stats.processed_successfully += results.len();
                }
                Err(e) => {
                    warn!("Error procesando paquetes de baja prioridad: {}", e);
                }
            }
        }

        stats.processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(HybridProcessingResult {
            processed_packages,
            failed_packages,
            stats,
        })
    }

    /// Clasificar paquete y determinar prioridad para API detalle
    async fn classify_and_prioritize_package(
        &self,
        package: &Package,
    ) -> Result<(DeliveryType, DetailPriority)> {
        // Aquí implementarías la lógica de clasificación
        // Por ahora, simplificado
        let delivery_type = DeliveryType::Domicile; // Placeholder
        
        let priority = match package.delivery_coordinates.as_ref() {
            Some(_) => DetailPriority::Low,
            None => DetailPriority::High,
        };

        Ok((delivery_type, priority))
    }

    /// Procesar paquete sin API detalle
    async fn process_without_detail(
        &self,
        package: Package,
        delivery_type: DeliveryType,
    ) -> Result<EnrichedPackage> {
        let enriched_data = EnrichedData {
            complete_address: package.delivery_address.clone(),
            coordinates: package.delivery_coordinates.as_ref().map(|coord| (coord.x, coord.y)),
            complete_barcode: package.external_tracking_number.clone(),
            physical_data: None,
            contact_info: None,
            delivery_instructions: None,
            tracking_history: None,
            special_comments: None,
        };

        Ok(EnrichedPackage {
            basic_data: package,
            detail_data: None,
            delivery_type,
            geocoding_quality: "Unknown".to_string(),
            detail_priority: DetailPriority::None,
            enriched_data,
        })
    }

    /// Procesar lote de paquetes con prioridad
    async fn process_priority_batch(
        &self,
        packages: Vec<(Package, DeliveryType)>,
        ref_colis_list: &[String],
        sso_token: &str,
    ) -> Result<Vec<EnrichedPackage>> {
        let mut results = Vec::new();
        
        // Intentar obtener del cache primero
        let cached_results = self.detail_cache.get_batch(ref_colis_list).await?;
        
        let mut packages_needing_detail = Vec::new();
        
        for (package, delivery_type) in packages.iter() {
            if let Some(Some(cached_response)) = cached_results.get(&package.tracking_number) {
                // Usar datos del cache
                match self.enrich_package(&package, Some(cached_response.clone()), delivery_type.clone()).await {
                    Ok(enriched) => results.push(enriched),
                    Err(e) => {
                        warn!("Error enriqueciendo paquete {} desde cache: {}", package.tracking_number, e);
                    }
                }
            } else {
                // Necesita llamada al API detalle
                packages_needing_detail.push((package.clone(), delivery_type.clone()));
            }
        }
        
        // Llamar al API detalle para paquetes que no están en cache
        if !packages_needing_detail.is_empty() {
            let ref_colis_needing_detail: Vec<String> = packages_needing_detail
                .iter()
                .map(|(p, _)| p.tracking_number.clone())
                .collect();
            
            match self.client.get_packages_detail_batch(&ref_colis_needing_detail, sso_token).await {
                Ok(detail_responses) => {
                    // Guardar en cache
                    for (ref_colis, response) in &detail_responses {
                        if response.success {
                            if let Err(e) = self.detail_cache.set(ref_colis, response.clone()).await {
                                warn!("Error guardando en cache: {}", e);
                            }
                        }
                    }
                    
                    // Procesar respuestas
                    for (package, delivery_type) in packages_needing_detail {
                        let detail_response = detail_responses.get(&package.tracking_number);
                        match self.enrich_package(&package, detail_response.cloned(), delivery_type).await {
                            Ok(enriched) => results.push(enriched),
                            Err(e) => {
                                warn!("Error enriqueciendo paquete {}: {}", package.tracking_number, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Error llamando API detalle: {}", e);
                    // Fallback: procesar sin datos detallados
                    for (package, delivery_type) in packages_needing_detail {
                        match self.process_without_detail(package, delivery_type).await {
                            Ok(enriched) => results.push(enriched),
                            Err(e) => {
                                warn!("Error en fallback: {}", e);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }

    /// Enriquecer paquete con datos detallados
    async fn enrich_package(
        &self,
        package: &Package,
        detail_response: Option<ColisDetailResponse>,
        delivery_type: DeliveryType,
    ) -> Result<EnrichedPackage> {
        let enriched_data = if let Some(ref detail) = detail_response {
            if detail.success {
                self.create_enriched_data(&package, &detail)?
            } else {
                self.create_basic_enriched_data(&package)?
            }
        } else {
            self.create_basic_enriched_data(&package)?
        };

        Ok(EnrichedPackage {
            basic_data: package.clone(),
            detail_data: detail_response,
            delivery_type,
            geocoding_quality: "Unknown".to_string(),
            detail_priority: DetailPriority::High, // Placeholder
            enriched_data,
        })
    }

    /// Crear datos enriquecidos con información detallada
    fn create_enriched_data(&self, package: &Package, detail: &ColisDetailResponse) -> Result<EnrichedData> {
        let mut enriched = self.create_basic_enriched_data(package)?;
        
        if let Some(detail_data) = &detail.data {
            // Enriquecer con datos detallados
            if let Some(complete_address) = &detail_data.adresse_complete {
                enriched.complete_address = complete_address.clone();
            }
            
            if let Some(coords) = &detail_data.coordonnees {
                enriched.coordinates = Some((coords.latitude, coords.longitude));
            }
            
            if let Some(barcode) = &detail_data.code_barre_complet {
                enriched.complete_barcode = Some(barcode.clone());
            }
            
            if let Some(physical) = &detail_data.donnees_physiques {
                enriched.physical_data = Some(PhysicalData {
                    weight: physical.poids,
                    weight_unit: physical.unite_poids.clone(),
                    dimensions: physical.dimensions.as_ref().map(|d| Dimensions {
                        length: d.longueur,
                        width: d.largeur,
                        height: d.hauteur,
                        unit: d.unite.clone(),
                    }),
                    value: physical.valeur,
                    currency: physical.devise.clone(),
                });
            }
            
            if let Some(contact) = &detail_data.contact {
                enriched.contact_info = Some(ContactInfo {
                    name: contact.nom.clone(),
                    phone: contact.telephone.clone(),
                    email: contact.email.clone(),
                });
            }
            
            if let Some(instructions) = &detail_data.instructions_livraison {
                enriched.delivery_instructions = Some(instructions.clone());
            }
            
            if let Some(history) = &detail_data.historique {
                enriched.tracking_history = Some(history.iter().map(|h| TrackingEvent {
                    date: h.date.clone(),
                    time: h.heure.clone(),
                    status: h.statut.clone(),
                    description: h.description.clone(),
                    location: h.lieu.clone(),
                }).collect());
            }
            
            if let Some(comments) = &detail_data.commentaires {
                enriched.special_comments = Some(comments.clone());
            }
        }
        
        Ok(enriched)
    }

    /// Crear datos enriquecidos básicos
    fn create_basic_enriched_data(&self, package: &Package) -> Result<EnrichedData> {
        Ok(EnrichedData {
            complete_address: package.delivery_address.clone(),
            coordinates: package.delivery_coordinates.as_ref().map(|coord| (coord.x, coord.y)),
            complete_barcode: package.external_tracking_number.clone(),
            physical_data: None,
            contact_info: None,
            delivery_instructions: None,
            tracking_history: None,
            special_comments: None,
        })
    }
}
