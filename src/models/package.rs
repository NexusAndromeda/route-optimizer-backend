use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Paquete individual de Colis Privé
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColisPrivePackage {
    pub code_barre_article: String,
    pub destinataire_nom: String,
    pub destinataire_telephone: Option<String>,
    pub destinataire_indication: Option<String>,
    pub num_voie_geocode_livraison: Option<String>,
    pub libelle_voie_geocode_livraison: String,
    pub code_postal_geocode_livraison: String,
    pub latitude: f64,
    pub longitude: f64,
    pub code_statut_article: Option<String>,
}

/// Paquete procesado con datos oficiales
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedPackage {
    pub id: Uuid,
    pub tracking: String,
    pub customer_name: String,
    pub phone_number: Option<String>,
    pub customer_indication: Option<String>,
    pub official_label: String,
    pub latitude: f64,
    pub longitude: f64,
    pub mailbox_access: bool,
    pub driver_notes: String,
    pub address_id: Option<Uuid>,
    pub code_statut_article: Option<String>,
}

/// Información de paquete para grupos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub id: Uuid,
    pub tracking: String,
    pub customer_indication: Option<String>,
}

/// Grupo de paquetes por cliente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerGroup {
    pub customer_name: String,
    pub phone_number: Option<String>,
    pub packages: Vec<PackageInfo>,
}

/// Grupo de entrega (múltiples paquetes en misma dirección)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryGroup {
    pub id: Uuid,
    pub official_label: String,
    pub latitude: f64,
    pub longitude: f64,
    pub mailbox_access: bool,
    pub driver_notes: String,
    pub customers: Vec<CustomerGroup>,
    pub total_packages: usize,
}

/// Paquete individual (1 paquete por dirección)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinglePackage {
    pub id: Uuid,
    pub tracking: String,
    pub customer_name: String,
    pub phone_number: Option<String>,
    pub customer_indication: Option<String>,
    pub official_label: String,
    pub latitude: f64,
    pub longitude: f64,
    pub mailbox_access: bool,
    pub driver_notes: String,
    pub address_id: Option<Uuid>,
    pub code_statut_article: Option<String>,
}

/// Respuesta estructurada con paquetes agrupados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupedPackages {
    pub singles: Vec<SinglePackage>,
    pub groups: Vec<DeliveryGroup>,
    pub total_packages: usize,
    pub total_addresses: usize,
}

impl GroupedPackages {
    pub fn new() -> Self {
        Self {
            singles: Vec::new(),
            groups: Vec::new(),
            total_packages: 0,
            total_addresses: 0,
        }
    }
    
    pub fn add_single(&mut self, package: SinglePackage) {
        self.singles.push(package);
        self.total_packages += 1;
        self.total_addresses += 1;
    }
    
    pub fn add_group(&mut self, group: DeliveryGroup) {
        self.total_packages += group.total_packages;
        self.total_addresses += 1;
        self.groups.push(group);
    }
    
    pub fn finalize(&mut self) {
        // Ordenar singles por tracking
        self.singles.sort_by(|a, b| a.tracking.cmp(&b.tracking));
        
        // Ordenar groups por official_label
        self.groups.sort_by(|a, b| a.official_label.cmp(&b.official_label));
        
        // Ordenar customers dentro de cada group
        for group in &mut self.groups {
            group.customers.sort_by(|a, b| a.customer_name.cmp(&b.customer_name));
            
            // Ordenar packages dentro de cada customer
            for customer in &mut group.customers {
                customer.packages.sort_by(|a, b| a.tracking.cmp(&b.tracking));
            }
        }
    }
}

impl From<ColisPrivePackage> for ProcessedPackage {
    fn from(colis: ColisPrivePackage) -> Self {
        Self {
            id: Uuid::new_v4(),
            tracking: colis.code_barre_article,
            customer_name: colis.destinataire_nom,
            phone_number: colis.destinataire_telephone,
            customer_indication: colis.destinataire_indication,
            official_label: format!("{} {} {}", 
                colis.num_voie_geocode_livraison.unwrap_or_default(),
                colis.libelle_voie_geocode_livraison,
                colis.code_postal_geocode_livraison
            ),
            latitude: colis.latitude,
            longitude: colis.longitude,
            mailbox_access: false,
            driver_notes: String::new(),
            address_id: None,
            code_statut_article: colis.code_statut_article,
        }
    }
}

impl From<ProcessedPackage> for SinglePackage {
    fn from(processed: ProcessedPackage) -> Self {
        Self {
            id: processed.id,
            tracking: processed.tracking,
            customer_name: processed.customer_name,
            phone_number: processed.phone_number,
            customer_indication: processed.customer_indication,
            official_label: processed.official_label,
            latitude: processed.latitude,
            longitude: processed.longitude,
            mailbox_access: processed.mailbox_access,
            driver_notes: processed.driver_notes,
            address_id: processed.address_id,
            code_statut_article: processed.code_statut_article,
        }
    }
}
