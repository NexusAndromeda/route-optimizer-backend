use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct TourneeInfo {
    #[serde(rename = "codeTourneeDistribution")]
    pub code_tournee_distribution: String,
    #[serde(rename = "nomDistributeur")]
    pub nom_distributeur: String,
    #[serde(rename = "matriculeDistributeur")]
    pub matricule_distributeur: String,
    #[serde(rename = "idSocieteDistributrice")]
    pub id_societe_distributrice: u32,
    #[serde(rename = "codeSocieteDistributrice")]
    pub code_societe_distributrice: String,
    #[serde(rename = "codeAgence")]
    pub code_agence: String,
    #[serde(rename = "codePays")]
    pub code_pays: String,
    #[serde(rename = "codeCentreCP")]
    pub code_centre_cp: String,
    #[serde(rename = "dateDebutTourneePrevue")]
    pub date_debut_tournee_prevue: String,
    #[serde(rename = "dateFinTourneeReelle")]
    pub date_fin_tournee_reelle: Option<String>,
    #[serde(rename = "VfEnDistri")]
    pub vf_en_distri: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageData {
    // Información básica
    #[serde(rename = "idLieuArticle")]
    pub id_lieu_article: Option<String>,
    #[serde(rename = "idArticle")]
    pub id_article: Option<String>,
    #[serde(rename = "codeTourneeMCP")]
    pub code_tournee_mcp: Option<String>,
    #[serde(rename = "codeTourneeDistribution")]
    pub code_tournee_distribution: Option<String>,
    #[serde(rename = "matriculeDistributeur")]
    pub matricule_distributeur: Option<String>,
    #[serde(rename = "nomDistributeur")]
    pub nom_distributeur: Option<String>,
    
    // Datos de dirección destinatario
    #[serde(rename = "LibelleVoieOrigineDestinataire")]
    pub libelle_voie_origine_destinataire: Option<String>,
    #[serde(rename = "LibelleLocaliteOrigineDestinataire")]
    pub libelle_localite_origine_destinataire: Option<String>,
    #[serde(rename = "codePostalOrigineDestinataire")]
    pub code_postal_origine_destinataire: Option<String>,
    #[serde(rename = "codePaysOrigineDestinataire")]
    pub code_pays_origine_destinataire: Option<String>,
    #[serde(rename = "complementAdresse1OrigineDestinataire")]
    pub complement_adresse_1_origine_destinataire: Option<String>,
    
    // Datos geocodificados destinatario
    #[serde(rename = "LibelleVoieGeocodeDestinataire")]
    pub libelle_voie_geocode_destinataire: Option<String>,
    #[serde(rename = "LibelleLocaliteGeocodeDestinataire")]
    pub libelle_localite_geocode_destinataire: Option<String>,
    #[serde(rename = "codePostalGeocodeDestinataire")]
    pub code_postal_geocode_destinataire: Option<String>,
    #[serde(rename = "codePaysGeocodeDestinataire")]
    pub code_pays_geocode_destinataire: Option<String>,
    #[serde(rename = "numVoieGeocodeDestinataire")]
    pub num_voie_geocode_destinataire: Option<String>,
    
    // Coordenadas GPS destinatario
    #[serde(rename = "coordXDestinataire")]
    pub coord_x_destinataire: Option<f64>,
    #[serde(rename = "coordYDestinataire")]
    pub coord_y_destinataire: Option<f64>,
    
    // Calidad del geocoding destinatario
    #[serde(rename = "qualiteGeocodageDestinataire")]
    pub qualite_geocodage_destinataire: Option<String>,
    #[serde(rename = "scoreGeocodageDestinataire")]
    pub score_geocodage_destinataire: Option<String>,
    #[serde(rename = "AlgoSolrDestinataire")]
    pub algo_solr_destinataire: Option<String>,
    
    // Datos de contacto destinatario
    #[serde(rename = "nomDestinataire")]
    pub nom_destinataire: Option<String>,
    #[serde(rename = "telephoneFixeDestinataire")]
    pub telephone_fixe_destinataire: Option<String>,
    #[serde(rename = "telephoneMobileDestinataire")]
    pub telephone_mobile_destinataire: Option<String>,
    #[serde(rename = "emailDestinataire")]
    pub email_destinataire: Option<String>,
    
    // Datos de dirección livraison
    #[serde(rename = "LibelleVoieOrigineLivraison")]
    pub libelle_voie_origine_livraison: Option<String>,
    #[serde(rename = "LibelleLocaliteOrigineLivraison")]
    pub libelle_localite_origine_livraison: Option<String>,
    #[serde(rename = "codePostalOrigineLivraison")]
    pub code_postal_origine_livraison: Option<String>,
    #[serde(rename = "codePaysOrigineLivraison")]
    pub code_pays_origine_livraison: Option<String>,
    
    // Datos geocodificados livraison
    #[serde(rename = "LibelleVoieGeocodeLivraison")]
    pub libelle_voie_geocode_livraison: Option<String>,
    #[serde(rename = "LibelleLocaliteGeocodeLivraison")]
    pub libelle_localite_geocode_livraison: Option<String>,
    #[serde(rename = "codePostalGeocodeLivraison")]
    pub code_postal_geocode_livraison: Option<String>,
    #[serde(rename = "codePaysGeocodeLivraison")]
    pub code_pays_geocode_livraison: Option<String>,
    #[serde(rename = "numVoieGeocodeLivraison")]
    pub num_voie_geocode_livraison: Option<String>,
    
    // Coordenadas GPS livraison
    #[serde(rename = "coordXLivraison")]
    pub coord_x_livraison: Option<f64>,
    #[serde(rename = "coordYLivraison")]
    pub coord_y_livraison: Option<f64>,
    
    // Calidad del geocoding livraison
    #[serde(rename = "qualiteGeocodageLivraison")]
    pub qualite_geocodage_livraison: Option<String>,
    #[serde(rename = "scoreGeocodageLivraison")]
    pub score_geocodage_livraison: Option<String>,
    #[serde(rename = "AlgoSolrLivraison")]
    pub algo_solr_livraison: Option<String>,
    
    // Datos de contacto livraison
    #[serde(rename = "nomLivraison")]
    pub nom_livraison: Option<String>,
    #[serde(rename = "telephoneFixeLivraison")]
    pub telephone_fixe_livraison: Option<String>,
    #[serde(rename = "telephoneMobileLivraison")]
    pub telephone_mobile_livraison: Option<String>,
    #[serde(rename = "mailLivraison")]
    pub mail_livraison: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TourneeResponse {
    #[serde(rename = "InfosTournee")]
    pub infos_tournee: TourneeInfo,
    #[serde(rename = "LstLieuArticle")]
    pub lst_lieu_article: Vec<PackageData>,
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
            
            if package.nom_destinataire.is_some() {
                report.packages_with_contact += 1;
            }
            
            if package.email_destinataire.is_some() {
                report.packages_with_email += 1;
            }
            
            if package.telephone_fixe_destinataire.is_some() || 
               package.telephone_mobile_destinataire.is_some() {
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
        let original_address = package.libelle_voie_origine_destinataire.clone().unwrap_or_else(|| "N/A".to_string());
        let geocoded_address = package.libelle_voie_geocode_destinataire.clone();
        let coordinates = match (package.coord_x_destinataire, package.coord_y_destinataire) {
            (Some(x), Some(y)) if x != 0.0 && y != 0.0 => Some((x, y)),
            _ => None,
        };
        
        let quality = package.qualite_geocodage_destinataire.clone();
        let algorithm = package.algo_solr_destinataire.clone();
        let score = package.score_geocodage_destinataire.clone();
        
        // Determinar estado del geocoding
        let status = self.determine_geocoding_status(package);
        
        // Detectar inconsistencias
        let has_inconsistencies = self.detect_inconsistencies(package);
        
        // Identificar campos faltantes
        let missing_fields = self.identify_missing_fields(package);
        
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
    
    fn determine_geocoding_status(&self, package: &PackageData) -> GeocodingStatus {
        let has_coordinates = package.coord_x_destinataire.is_some() && package.coord_y_destinataire.is_some();
        let has_geocoded_address = package.libelle_voie_geocode_destinataire.is_some();
        let has_quality = package.qualite_geocodage_destinataire.is_some();
        
        if has_coordinates && has_geocoded_address && has_quality {
            if let Some(quality) = &package.qualite_geocodage_destinataire {
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
    
    fn detect_inconsistencies(&self, package: &PackageData) -> bool {
        // Verificar inconsistencias entre datos originales y geocodificados
        let voie_inconsistency = package.libelle_voie_origine_destinataire != package.libelle_voie_geocode_destinataire;
        let postal_inconsistency = package.code_postal_origine_destinataire != package.code_postal_geocode_destinataire;
        let localite_inconsistency = package.libelle_localite_origine_destinataire != package.libelle_localite_geocode_destinataire;
        
        voie_inconsistency || postal_inconsistency || localite_inconsistency
    }
    
    fn identify_missing_fields(&self, package: &PackageData) -> Vec<String> {
        let mut missing = Vec::new();
        
        if package.libelle_voie_origine_destinataire.is_none() {
            missing.push("libelle_voie_origine_destinataire".to_string());
        }
        if package.coord_x_destinataire.is_none() {
            missing.push("coord_x_destinataire".to_string());
        }
        if package.coord_y_destinataire.is_none() {
            missing.push("coord_y_destinataire".to_string());
        }
        if package.qualite_geocodage_destinataire.is_none() {
            missing.push("qualite_geocodage_destinataire".to_string());
        }
        if package.algo_solr_destinataire.is_none() {
            missing.push("algo_solr_destinataire".to_string());
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Uso: {} <ruta_al_json>", args[0]);
        eprintln!("Ejemplo: {} ../apk_analysis/response-getTournéeByMatricule-3.json", args[0]);
        std::process::exit(1);
    }
    
    let json_path = &args[1];
    println!("Analizando JSON: {}", json_path);
    
    // Cargar y analizar el JSON
    let tournee = TourneeResponse::from_file(json_path)?;
    let report = tournee.analyze_geocoding();
    
    // Mostrar resumen
    report.print_summary();
    
    // Mostrar recomendaciones detalladas
    println!("\n=== RECOMENDACIONES DETALLADAS ===");
    for (i, rec) in report.recommendations.iter().enumerate() {
        if i < 10 { // Mostrar solo los primeros 10
            println!("Paquete {}: {:?} - {:?}", rec.package_id, rec.status, rec.recommendation);
            if rec.has_inconsistencies {
                println!("  ⚠️  Inconsistencias detectadas");
            }
            if !rec.missing_fields.is_empty() {
                println!("  ❌ Campos faltantes: {:?}", rec.missing_fields);
            }
        }
    }
    
    if report.recommendations.len() > 10 {
        println!("... y {} paquetes más", report.recommendations.len() - 10);
    }
    
    // Guardar reporte completo en JSON
    let report_json = serde_json::to_string_pretty(&report)?;
    std::fs::write("geocoding_analysis_report.json", report_json)?;
    println!("\nReporte completo guardado en: geocoding_analysis_report.json");
    
    Ok(())
}
