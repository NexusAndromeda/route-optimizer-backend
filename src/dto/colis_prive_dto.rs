use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// Re-export para compatibilidad
pub use crate::dto::colis_prive_dto::PackageData as PublicPackageData;

// Request para autenticación Colis Privé
#[derive(Debug, Deserialize)]
pub struct ColisPriveAuthRequest {
    pub username: String,
    pub password: String,
    pub societe: String,
}

// Response de autenticación Colis Privé
#[derive(Debug, Serialize)]
pub struct ColisPriveAuthResponse {
    pub success: bool,
    pub message: Option<String>,
    pub authentication: Option<ColisPriveAuthData>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ColisPriveAuthData {
    pub sso_token: String,
    pub matricule_chauffeur: String,
    pub nom_chauffeur: String,
    pub societe: String,
    pub expires_at: DateTime<Utc>,
}

// Request para obtener paquetes
#[derive(Debug, Deserialize)]
pub struct GetPackagesRequest {
    pub matricule: String,
    pub societe: String,
    pub date: Option<String>,
}

// Response de paquetes
#[derive(Debug, Serialize)]
pub struct PackagesResponse {
    pub success: bool,
    pub packages: Vec<PackageData>,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PackageData {
    // Campos principales de Colis Privé
    pub reference_colis: String,
    pub destinataire_nom: String,
    pub destinataire_adresse1: Option<String>,
    pub destinataire_adresse2: Option<String>,
    pub destinataire_cp: Option<String>,
    pub destinataire_ville: Option<String>,
    pub coord_x_destinataire: Option<f64>,
    pub coord_y_destinataire: Option<f64>,
    pub statut: Option<String>,
    pub numero_ordre: Option<i32>,
    
    // Campos legacy para compatibilidad
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_fixed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_warnings: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ordre_passage_prevu: Option<i32>,
}

// Request para optimización
#[derive(Debug, Deserialize)]
pub struct OptimizeRouteRequest {
    pub matricule: String,
    pub societe: String,
}

// Response de optimización
#[derive(Debug, Serialize)]
pub struct OptimizeRouteResponse {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<OptimizationData>,
}

#[derive(Debug, Serialize)]
pub struct OptimizationData {
    pub matricule_chauffeur: String,
    pub date_tournee: String,
    pub optimized_packages: Vec<PackageData>,
}

// Company list response
#[derive(Debug, Serialize)]
pub struct CompaniesListResponse {
    pub success: bool,
    pub companies: Vec<CompanyInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompanyInfo {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
}

