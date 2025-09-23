use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct TourneeInfo {
    pub code_tournee_distribution: String,
    pub nom_distributeur: String,
    pub matricule_distributeur: String,
    pub id_societe_distributrice: u32,
    pub code_societe_distributrice: String,
    pub code_agence: String,
    pub code_pays: String,
    pub code_centre_cp: String,
    pub date_debut_tournee_prevue: String,
    pub date_fin_tournee_reelle: Option<String>,
    pub vf_en_distri: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressData {
    // Datos originales
    pub libelle_voie_origine: Option<String>,
    pub libelle_localite_origine: Option<String>,
    pub code_postal_origine: Option<String>,
    pub code_pays_origine: Option<String>,
    pub complement_adresse_1: Option<String>,
    pub complement_adresse_2: Option<String>,
    pub complement_adresse_3: Option<String>,
    
    // Datos geocodificados
    pub libelle_voie_geocode: Option<String>,
    pub libelle_localite_geocode: Option<String>,
    pub code_postal_geocode: Option<String>,
    pub code_pays_geocode: Option<String>,
    pub num_voie_geocode: Option<String>,
    
    // Coordenadas GPS
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    
    // Calidad del geocoding
    pub qualite_geocodage: Option<String>,
    pub score_geocodage: Option<String>,
    pub algo_solr: Option<String>,
    
    // IDs de geocoding
    pub id_adresse_base: Option<String>,
    pub hvv_id_adresse: Option<String>,
    pub id_adresse_ign: Option<String>,
    pub hvv_matricule_voie: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactData {
    pub nom: Option<String>,
    pub telephone_fixe: Option<String>,
    pub telephone_mobile: Option<String>,
    pub email: Option<String>,
    pub commentaire: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeliveryData {
    pub type_livraison: Option<String>,
    pub type_loc: Option<String>,
    pub date_livraison: Option<String>,
    pub date_premiere_livraison_possible: Option<String>,
    pub ref_cab: Option<String>,
    pub horaires: Option<String>,
    pub digicode: Option<String>,
    pub cote: Option<String>,
    pub commentaire: Option<String>,
    pub code_ug: Option<String>,
    pub code_ug_technique: Option<String>,
    pub code_iris: Option<String>,
    pub code_realis: Option<String>,
    pub id_troncon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleData {
    pub metier: Option<String>,
    pub code_pays: Option<String>,
    pub info1: Option<String>,
    pub info2: Option<String>,
    pub preference_livraison: Option<String>,
    pub ref_externe: Option<String>,
    pub code_barre: Option<String>,
    pub code_societe_emetrice: Option<String>,
    pub code_societe_prise_en_charge: Option<String>,
    pub type_mise_dispo_colis: Option<String>,
    pub co_origine_creation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionData {
    pub id_action: Option<String>,
    pub code_cle_action: Option<String>,
    pub code_action: Option<String>,
    pub libelle_action: Option<String>,
    pub code_type_action: Option<String>,
    pub num_ordre_action: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompteurRenduData {
    pub id_cpt_rendu: Option<String>,
    pub code_type_cpt_rendu: Option<String>,
    pub valeur_attendu: Option<String>,
    pub valeur: Option<String>,
    pub horodatage: Option<String>,
    pub coord_x_gps: Option<f64>,
    pub coord_y_gps: Option<f64>,
    pub gps_qualite: Option<String>,
    pub vf_transmis_si_tiers: Option<bool>,
    pub date_transmis_si_tiers: Option<String>,
    pub num_ordre: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageData {
    // Información básica
    pub id_lieu_article: Option<String>,
    pub id_article: Option<String>,
    pub code_tournee_mcp: Option<String>,
    pub code_tournee_distribution: Option<String>,
    pub matricule_distributeur: Option<String>,
    pub nom_distributeur: Option<String>,
    
    // Datos de dirección
    pub destinataire: AddressData,
    pub livraison: AddressData,
    
    // Datos de contacto
    pub contact_destinataire: ContactData,
    pub contact_livraison: ContactData,
    
    // Datos de entrega
    pub delivery: DeliveryData,
    
    // Datos del artículo
    pub article: ArticleData,
    
    // Datos de acción
    pub action: ActionData,
    
    // Datos de contador
    pub compteur_rendu: CompteurRenduData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TourneeResponse {
    pub infos_tournee: TourneeInfo,
    pub lst_lieu_article: Vec<PackageData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeocodingAnalysis {
    pub package_id: String,
    pub status: GeocodingStatus,
    pub original_address: String,
    pub geocoded_address: Option<String>,
    pub coordinates: Option<(f64, f64)>,
    pub quality: Option<String>,
    pub algorithm: Option<String>,
    pub score: Option<String>,
    pub has_inconsistencies: bool,
    pub missing_fields: Vec<String>,
    pub recommendation: Recommendation,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GeocodingStatus {
    Complete,
    Partial,
    Failed,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Recommendation {
    UseExisting,
    ValidateCoords,
    ReGeocode,
    ManualReview,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub total_packages: usize,
    pub geocoding_complete: usize,
    pub geocoding_partial: usize,
    pub geocoding_failed: usize,
    pub geocoding_unknown: usize,
    pub packages_with_coordinates: usize,
    pub packages_with_contact: usize,
    pub packages_with_email: usize,
    pub packages_with_phone: usize,
    pub quality_distribution: HashMap<String, usize>,
    pub algorithm_distribution: HashMap<String, usize>,
    pub recommendations: Vec<GeocodingAnalysis>,
}

impl TourneeResponse {
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let response: TourneeResponse = serde_json::from_str(&content)?;
        Ok(response)
    }
    
    pub fn analyze_geocoding(&self) -> AnalysisReport {
        let mut report = AnalysisReport {
            total_packages: self.lst_lieu_article.len(),
            geocoding_complete: 0,
            geocoding_partial: 0,
            geocoding_failed: 0,
            geocoding_unknown: 0,
            packages_with_coordinates: 0,
            packages_with_contact: 0,
            packages_with_email: 0,
            packages_with_phone: 0,
            quality_distribution: HashMap::new(),
            algorithm_distribution: HashMap::new(),
            recommendations: Vec::new(),
        };
        
        for package in &self.lst_lieu_article {
            let analysis = self.analyze_package_geocoding(package);
            
            match analysis.status {
                GeocodingStatus::Complete => report.geocoding_complete += 1,
                GeocodingStatus::Partial => report.geocoding_partial += 1,
                GeocodingStatus::Failed => report.geocoding_failed += 1,
                GeocodingStatus::Unknown => report.geocoding_unknown += 1,
            }
            
            if analysis.coordinates.is_some() {
                report.packages_with_coordinates += 1;
            }
            
            if package.contact_destinataire.nom.is_some() {
                report.packages_with_contact += 1;
            }
            
            if package.contact_destinataire.email.is_some() {
                report.packages_with_email += 1;
            }
            
            if package.contact_destinataire.telephone_fixe.is_some() || 
               package.contact_destinataire.telephone_mobile.is_some() {
                report.packages_with_phone += 1;
            }
            
            if let Some(quality) = &analysis.quality {
                *report.quality_distribution.entry(quality.clone()).or_insert(0) += 1;
            }
            
            if let Some(algorithm) = &analysis.algorithm {
                *report.algorithm_distribution.entry(algorithm.clone()).or_insert(0) += 1;
            }
            
            report.recommendations.push(analysis);
        }
        
        report
    }
    
    fn analyze_package_geocoding(&self, package: &PackageData) -> GeocodingAnalysis {
        let package_id = package.id_lieu_article.clone().unwrap_or_else(|| "unknown".to_string());
        
        // Analizar datos de destinatario
        let destinataire = &package.destinataire;
        let original_address = destinataire.libelle_voie_origine.clone().unwrap_or_else(|| "N/A".to_string());
        let geocoded_address = destinataire.libelle_voie_geocode.clone();
        let coordinates = match (destinataire.coord_x, destinataire.coord_y) {
            (Some(x), Some(y)) if x != 0.0 && y != 0.0 => Some((x, y)),
            _ => None,
        };
        
        let quality = destinataire.qualite_geocodage.clone();
        let algorithm = destinataire.algo_solr.clone();
        let score = destinataire.score_geocodage.clone();
        
        // Determinar estado del geocoding
        let status = self.determine_geocoding_status(destinataire);
        
        // Detectar inconsistencias
        let has_inconsistencies = self.detect_inconsistencies(destinataire);
        
        // Identificar campos faltantes
        let missing_fields = self.identify_missing_fields(destinataire);
        
        // Generar recomendación
        let recommendation = self.generate_recommendation(&status, &coordinates, &has_inconsistencies);
        
        GeocodingAnalysis {
            package_id,
            status,
            original_address,
            geocoded_address,
            coordinates,
            quality,
            algorithm,
            score,
            has_inconsistencies,
            missing_fields,
            recommendation,
        }
    }
    
    fn determine_geocoding_status(&self, address: &AddressData) -> GeocodingStatus {
        let has_coordinates = address.coord_x.is_some() && address.coord_y.is_some();
        let has_geocoded_address = address.libelle_voie_geocode.is_some();
        let has_quality = address.qualite_geocodage.is_some();
        
        if has_coordinates && has_geocoded_address && has_quality {
            if let Some(quality) = &address.qualite_geocodage {
                match quality.as_str() {
                    "Bon" => GeocodingStatus::Complete,
                    "Moyen" => GeocodingStatus::Partial,
                    "Mauvais" => GeocodingStatus::Failed,
                    _ => GeocodingStatus::Unknown,
                }
            } else {
                GeocodingStatus::Unknown
            }
        } else if has_coordinates || has_geocoded_address {
            GeocodingStatus::Partial
        } else {
            GeocodingStatus::Failed
        }
    }
    
    fn detect_inconsistencies(&self, address: &AddressData) -> bool {
        // Verificar inconsistencias entre datos originales y geocodificados
        let voie_inconsistency = address.libelle_voie_origine != address.libelle_voie_geocode;
        let postal_inconsistency = address.code_postal_origine != address.code_postal_geocode;
        let localite_inconsistency = address.libelle_localite_origine != address.libelle_localite_geocode;
        
        voie_inconsistency || postal_inconsistency || localite_inconsistency
    }
    
    fn identify_missing_fields(&self, address: &AddressData) -> Vec<String> {
        let mut missing = Vec::new();
        
        if address.libelle_voie_origine.is_none() {
            missing.push("libelle_voie_origine".to_string());
        }
        if address.coord_x.is_none() {
            missing.push("coord_x".to_string());
        }
        if address.coord_y.is_none() {
            missing.push("coord_y".to_string());
        }
        if address.qualite_geocodage.is_none() {
            missing.push("qualite_geocodage".to_string());
        }
        if address.algo_solr.is_none() {
            missing.push("algo_solr".to_string());
        }
        
        missing
    }
    
    fn generate_recommendation(&self, status: &GeocodingStatus, coordinates: &Option<(f64, f64)>, has_inconsistencies: &bool) -> Recommendation {
        match status {
            GeocodingStatus::Complete => {
                if *has_inconsistencies {
                    Recommendation::ValidateCoords
                } else {
                    Recommendation::UseExisting
                }
            },
            GeocodingStatus::Partial => {
                if coordinates.is_some() {
                    Recommendation::ValidateCoords
                } else {
                    Recommendation::ReGeocode
                }
            },
            GeocodingStatus::Failed => Recommendation::ReGeocode,
            GeocodingStatus::Unknown => Recommendation::ManualReview,
        }
    }
}

impl AnalysisReport {
    pub fn print_summary(&self) {
        println!("=== ANÁLISIS DE GEOCODING ===");
        println!("Total de paquetes: {}", self.total_packages);
        println!("Geocoding completo: {} ({:.1}%)", 
                self.geocoding_complete, 
                (self.geocoding_complete as f64 / self.total_packages as f64) * 100.0);
        println!("Geocoding parcial: {} ({:.1}%)", 
                self.geocoding_partial, 
                (self.geocoding_partial as f64 / self.total_packages as f64) * 100.0);
        println!("Geocoding fallido: {} ({:.1}%)", 
                self.geocoding_failed, 
                (self.geocoding_failed as f64 / self.total_packages as f64) * 100.0);
        println!("Con coordenadas: {} ({:.1}%)", 
                self.packages_with_coordinates, 
                (self.packages_with_coordinates as f64 / self.total_packages as f64) * 100.0);
        println!("Con contacto: {} ({:.1}%)", 
                self.packages_with_contact, 
                (self.packages_with_contact as f64 / self.total_packages as f64) * 100.0);
        println!("Con email: {} ({:.1}%)", 
                self.packages_with_email, 
                (self.packages_with_email as f64 / self.total_packages as f64) * 100.0);
        println!("Con teléfono: {} ({:.1}%)", 
                self.packages_with_phone, 
                (self.packages_with_phone as f64 / self.total_packages as f64) * 100.0);
        
        println!("\n=== DISTRIBUCIÓN DE CALIDAD ===");
        for (quality, count) in &self.quality_distribution {
            println!("{}: {} ({:.1}%)", 
                    quality, 
                    count, 
                    (*count as f64 / self.total_packages as f64) * 100.0);
        }
        
        println!("\n=== DISTRIBUCIÓN DE ALGORITMOS ===");
        for (algorithm, count) in &self.algorithm_distribution {
            println!("{}: {} ({:.1}%)", 
                    algorithm, 
                    count, 
                    (*count as f64 / self.total_packages as f64) * 100.0);
        }
    }
}
