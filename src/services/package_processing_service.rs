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
        let mut processed = ProcessedPackage::from(colis_package.clone());
        
        // Crear ColisPriveAddress para matching
        let colis_addr = ColisPriveAddress {
            num_voie: colis_package.num_voie_geocode_livraison.clone(),
            libelle_voie: colis_package.libelle_voie_geocode_livraison.clone(),
            code_postal: colis_package.code_postal_geocode_livraison.clone(),
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
                
                info!("✅ Match encontrado para {}: {}", 
                    colis_package.libelle_voie_geocode_livraison, 
                    official_address.official_label);
            }
            None => {
                // ❌ NO MATCH - crear nueva dirección o usar datos de Colis Privé
                match self.address_matcher.create_address_if_not_exists(
                    colis_package.libelle_voie_geocode_livraison.clone(),
                    colis_package.code_postal_geocode_livraison.clone(),
                    "Paris".to_string(), // Asumimos París por ahora
                    colis_package.latitude,
                    colis_package.longitude,
                    company_id,
                ).await {
                    Ok(new_address) => {
                        let official_label = new_address.official_label.clone();
                        processed.official_label = new_address.official_label;
                        processed.latitude = new_address.latitude;
                        processed.longitude = new_address.longitude;
                        processed.mailbox_access = new_address.has_mailbox_access;
                        processed.driver_notes = new_address.driver_notes.unwrap_or_default();
                        processed.address_id = Some(new_address.id);
                        
                        info!("🆕 Nueva dirección creada: {}", official_label);
                    }
                    Err(e) => {
                        warn!("⚠️ Error creando dirección, usando datos de Colis Privé: {}", e);
                        // Los datos ya están en processed desde el From impl
                    }
                }
            }
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
