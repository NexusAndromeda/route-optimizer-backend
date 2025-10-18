use crate::models::package::{
    ColisPrivePackage, ProcessedPackage, GroupedPackages, 
    SinglePackage, DeliveryGroup, CustomerGroup, PackageInfo
};
use crate::models::address::ColisPriveAddress;
use crate::services::address_matching_service::AddressMatchingService;
use std::collections::HashMap;
use anyhow::Result;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Servicio para procesar y agrupar paquetes de Colis Privé
pub struct PackageProcessingService {
    address_matcher: AddressMatchingService,
}

impl PackageProcessingService {
    pub fn new(address_matcher: AddressMatchingService) -> Self {
        Self { address_matcher }
    }
    
    /// Procesa una tournée de paquetes de Colis Privé y los agrupa
    pub async fn process_tournee(
        &self, 
        packages: Vec<ColisPrivePackage>,
        company_id: Option<Uuid>
    ) -> Result<GroupedPackages> {
        info!("🔄 Procesando {} paquetes de Colis Privé", packages.len());
        
        let mut grouped = GroupedPackages::new();
        let mut processed_packages = Vec::new();
        
        // 1. Procesar cada paquete individualmente
        for colis_package in packages {
            let tracking = colis_package.code_barre_article.clone();
            match self.process_single_package(colis_package, company_id).await {
                Ok(processed) => {
                    processed_packages.push(processed);
                }
                Err(e) => {
                    error!("❌ Error procesando paquete {}: {}", tracking, e);
                }
            }
        }
        
        info!("✅ {} paquetes procesados exitosamente", processed_packages.len());
        
        // 2. Agrupar por dirección física
        self.group_packages_by_address(processed_packages, &mut grouped).await?;
        
        // 3. Finalizar y ordenar
        grouped.finalize();
        
        info!("📊 Agrupación completada: {} singles, {} groups, {} paquetes totales", 
            grouped.singles.len(), grouped.groups.len(), grouped.total_packages);
        
        Ok(grouped)
    }
    
    /// Procesa un paquete individual con matching de dirección
    async fn process_single_package(
        &self, 
        colis_package: ColisPrivePackage,
        company_id: Option<Uuid>
    ) -> Result<ProcessedPackage> {
        let tracking = colis_package.code_barre_article.clone();
        
        // PASO 1: Verificar calidad de geocodificación
        let qualite = colis_package.qualite_geocodage_destinataire.as_deref().unwrap_or("");
        
        let (libelle_voie, code_postal, num_voie, is_problematic) = if qualite == "Bon" {
            // ✅ Calidad "Bon" - usar GeocodeDestinataire
            let libelle = colis_package.libelle_voie_geocode_destinataire.clone()
                .unwrap_or_default();
            let cp = colis_package.code_postal_geocode_destinataire.clone()
                .unwrap_or_default();
            let num = colis_package.num_voie_geocode_destinataire.clone();
            
            info!("✅ Calidad 'Bon' para {}: {} {}", tracking, num.as_deref().unwrap_or(""), libelle);
            (libelle, cp, num, false)
        } else {
            // 🚨 Calidad "Mauvais" - marcar como problemático
            warn!("🚨 Calidad '{}' para {}, marcando como problemático", qualite, tracking);
            
            // Usar OrigineDestinataire como fallback visual
            let libelle = colis_package.libelle_voie_origine_destinataire.clone()
                .unwrap_or_default();
            let cp = colis_package.code_postal_origine_destinataire.clone()
                .unwrap_or_default();
            
            (libelle, cp, None, true)
        };
        
        // PASO 2: Extraer número si está incluido en libelle
        let numero_final = if let Some(num) = num_voie {
            num
        } else if let Some(captures) = regex::Regex::new(r"^(\d+[A-Z]?)\s+(.+)").unwrap().captures(&libelle_voie) {
            captures.get(1).map(|m| m.as_str().to_string()).unwrap_or_default()
        } else {
            String::new()
        };
        
        // PASO 3: Construir official_label
        let official_label = if !numero_final.is_empty() {
            format!("{} {} {}", numero_final, libelle_voie, code_postal)
        } else {
            format!("{} {}", libelle_voie, code_postal)
        }.trim().to_string();
        
        // PASO 4: Crear ProcessedPackage
        let mut processed = ProcessedPackage {
            id: Uuid::new_v4(),
            tracking: tracking.clone(),
            customer_name: colis_package.destinataire_nom,
            phone_number: colis_package.destinataire_telephone,
            customer_indication: colis_package.destinataire_indication,
            official_label: official_label.clone(),
            latitude: colis_package.latitude,
            longitude: colis_package.longitude,
            mailbox_access: false,
            driver_notes: String::new(),
            address_id: None,
            code_statut_article: colis_package.code_statut_article,
            is_problematic,
        };
        
        // PASO 5: Si NO es problemático, intentar matching con BD
        if !is_problematic {
            let colis_addr = ColisPriveAddress {
                num_voie: Some(numero_final.clone()).filter(|s| !s.is_empty()),
                libelle_voie: libelle_voie.clone(),
                code_postal: code_postal.clone(),
                latitude: colis_package.latitude,
                longitude: colis_package.longitude,
            };
            
            // Buscar dirección oficial
            match self.address_matcher.find_colis_prive_address(&colis_addr).await {
            Some(official_address) => {
                // ✅ MATCH ENCONTRADO - usar datos oficiales
                processed.official_label = official_address.official_label.clone();
                processed.latitude = official_address.latitude;
                processed.longitude = official_address.longitude;
                processed.mailbox_access = official_address.has_mailbox_access;
                processed.driver_notes = official_address.driver_notes.unwrap_or_default();
                processed.address_id = Some(official_address.id);
                
                info!("✅ Match BD encontrado para {}: {}", 
                    tracking, 
                    official_address.official_label);
            }
            None => {
                // ❌ NO MATCH - crear nueva dirección en BD
                match self.address_matcher.create_address_if_not_exists(
                    libelle_voie.clone(),
                    code_postal.clone(),
                    "Paris".to_string(),
                    processed.latitude,
                    processed.longitude,
                    company_id,
                ).await {
                    Ok(new_address) => {
                        processed.official_label = new_address.official_label.clone();
                        processed.latitude = new_address.latitude;
                        processed.longitude = new_address.longitude;
                        processed.mailbox_access = new_address.has_mailbox_access;
                        processed.driver_notes = new_address.driver_notes.unwrap_or_default();
                        processed.address_id = Some(new_address.id);
                        
                        info!("🆕 Nueva dirección creada en BD: {}", new_address.official_label);
                    }
                    Err(e) => {
                        warn!("⚠️ Error creando dirección para {}: {}", tracking, e);
                    }
                }
            }
            } // Cerrar el match
        } else {
            info!("🚨 Paquete {} marcado como problemático (qualite != 'Bon')", tracking);
        }
        
        Ok(processed)
    }
    
    /// Agrupa paquetes por dirección física
    async fn group_packages_by_address(
        &self,
        packages: Vec<ProcessedPackage>,
        grouped: &mut GroupedPackages,
    ) -> Result<()> {
        let mut address_groups: HashMap<String, Vec<ProcessedPackage>> = HashMap::new();
        
        // Agrupar por dirección física
        for package in packages {
            let address_key = package.official_label.clone();
            address_groups.entry(address_key).or_insert_with(Vec::new).push(package);
        }
        
        // Procesar cada grupo de dirección
        for (_address, packages) in address_groups {
            if packages.len() == 1 {
                // Single package
                let single = SinglePackage::from(packages.into_iter().next().unwrap());
                grouped.add_single(single);
            } else {
                // Multiple packages - crear grupo
                let group = self.create_delivery_group(packages).await?;
                grouped.add_group(group);
            }
        }
        
        Ok(())
    }
    
    /// Crea un grupo de entrega con múltiples paquetes
    async fn create_delivery_group(
        &self,
        packages: Vec<ProcessedPackage>,
    ) -> Result<DeliveryGroup> {
        if packages.is_empty() {
            return Err(anyhow::anyhow!("No packages provided for group"));
        }
        
        // Guardar información del primer paquete antes de mover packages
        let first_package_info = {
            let pkg = &packages[0];
            (pkg.official_label.clone(), pkg.latitude, pkg.longitude, 
             pkg.mailbox_access, pkg.driver_notes.clone())
        };
        let total_packages = packages.len();
        
        // Agrupar por cliente
        let mut customer_groups: HashMap<String, Vec<ProcessedPackage>> = HashMap::new();
        for package in packages {
            customer_groups.entry(package.customer_name.clone())
                .or_insert_with(Vec::new)
                .push(package);
        }
        
        // Convertir a CustomerGroup
        let mut customers = Vec::new();
        for (customer_name, customer_packages) in customer_groups {
            // Guardar phone_number antes de iterar
            let phone_number = customer_packages[0].phone_number.clone();
            
            let packages_info: Vec<PackageInfo> = customer_packages.into_iter()
                .map(|pkg| PackageInfo {
                    id: pkg.id,
                    tracking: pkg.tracking,
                    customer_indication: pkg.customer_indication,
                })
                .collect();
            
            customers.push(CustomerGroup {
                customer_name,
                phone_number,
                packages: packages_info,
            });
        }
        
        Ok(DeliveryGroup {
            id: Uuid::new_v4(),
            official_label: first_package_info.0,
            latitude: first_package_info.1,
            longitude: first_package_info.2,
            mailbox_access: first_package_info.3,
            driver_notes: first_package_info.4,
            customers,
            total_packages,
        })
    }
    
    /// Obtiene estadísticas del procesamiento
    pub async fn get_processing_stats(&self) -> Result<(usize, Vec<String>)> {
        self.address_matcher.get_cache_stats().await
    }
}
