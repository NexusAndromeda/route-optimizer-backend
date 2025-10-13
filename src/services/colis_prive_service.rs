use crate::config::environment::EnvironmentConfig;
use crate::dto::colis_prive_dto;
use crate::utils::errors::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

// Re-exports para compatibilidad con código legacy
pub use crate::dto::colis_prive_dto::PackageData;

// Estructuras legacy para compatibilidad
#[derive(Debug, Serialize, Deserialize)]
pub struct ColisPriveAuthRequest {
    pub username: String,
    pub password: String,
    pub societe: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColisPriveAuthResponse {
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
    pub matricule: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTourneeRequest {
    pub username: String,
    pub password: String,
    pub societe: String,
    pub matricule: String,
    pub date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPackagesRequest {
    pub matricule: String,
    pub societe: String,
    pub date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPackagesResponse {
    pub success: bool,
    pub packages: Vec<PackageData>,
    pub total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_validation: Option<AddressValidationSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressValidationSummary {
    pub total_packages: usize,
    pub with_coordinates: usize,
    pub without_coordinates: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_validated: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleaned_auto: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_auto: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_found: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geocoding_errors: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_manual: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct AuthApiRequest {
    #[serde(rename = "identifiant")]
    identifiant: String,
    #[serde(rename = "motDePasse")]
    mot_de_passe: String,
}

#[derive(Debug, Deserialize)]
struct AuthApiResponse {
    #[serde(rename = "Success")]
    success: bool,
    #[serde(rename = "Message")]
    message: Option<String>,
    #[serde(rename = "Data")]
    data: Option<AuthApiData>,
}

#[derive(Debug, Deserialize)]
struct AuthApiData {
    #[serde(rename = "SsoHopps")]
    sso_hopps: String,
    #[serde(rename = "matriculeChauffeur")]
    matricule_chauffeur: String,
    #[serde(rename = "nomChauffeur")]
    nom_chauffeur: String,
}

#[derive(Debug, Serialize)]
struct TourneeApiRequest {
    #[serde(rename = "codeSociete")]
    code_societe: String,
    matricule: String,
    #[serde(rename = "dateHeureDebut")]
    date_heure_debut: String,
}

#[derive(Debug, Deserialize)]
struct TourneeApiResponse {
    #[serde(rename = "Success")]
    success: bool,
    #[serde(rename = "Data")]
    data: Option<TourneeApiData>,
}

#[derive(Debug, Deserialize)]
struct TourneeApiData {
    #[serde(rename = "lstLieuArticle")]
    lst_lieu_article: Vec<LieuArticle>,
}

#[derive(Debug, Deserialize)]
struct LieuArticle {
    #[serde(rename = "numeroOrdre")]
    numero_ordre: Option<i32>,
    #[serde(rename = "referenceColis")]
    reference_colis: Option<String>,
    #[serde(rename = "destinataireNom")]
    destinataire_nom: Option<String>,
    #[serde(rename = "destinataireAdresse1")]
    destinataire_adresse1: Option<String>,
    #[serde(rename = "destinataireAdresse2")]
    destinataire_adresse2: Option<String>,
    #[serde(rename = "destinataireCp")]
    destinataire_cp: Option<String>,
    #[serde(rename = "destinataireVille")]
    destinataire_ville: Option<String>,
    #[serde(rename = "coordXDestinataire")]
    coord_x_destinataire: Option<f64>,
    #[serde(rename = "coordYDestinataire")]
    coord_y_destinataire: Option<f64>,
    statut: Option<String>,
}

pub struct ColisPriveService {
    client: Client,
    config: EnvironmentConfig,
}

pub struct AuthenticationResult {
    pub sso_token: String,
    pub matricule_chauffeur: String,
    pub nom_chauffeur: String,
    pub expires_at: DateTime<Utc>,
}

pub struct OptimizationResult {
    pub matricule_chauffeur: String,
    pub date_tournee: String,
    pub packages: Vec<PackageData>,
}

impl ColisPriveService {
    pub fn new(client: Client, config: EnvironmentConfig) -> Self {
        Self { client, config }
    }

    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
        societe: &str,
    ) -> Result<AuthenticationResult, AppError> {
        let full_username = format!("{}_{}", societe, username);

        let auth_request = AuthApiRequest {
            identifiant: full_username.clone(),
            mot_de_passe: password.to_string(),
        };

        log::info!("🔗 Conectando a wsauth.colisprive.com...");
        
        let response = self.client
            .post("https://wsauth.colisprive.com/WS-AuthColis/api/authColisPriveLogin_POST/")
            .json(&auth_request)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| {
                log::error!("❌ Error de conexión: {:?}", e);
                AppError::ExternalApi(format!("Error en request de autenticación: {}", e))
            })?;
        
        log::info!("✅ Respuesta recibida: {}", response.status());

        if !response.status().is_success() {
            return Err(AppError::Unauthorized(format!("Error de autenticación: {}", response.status())));
        }

        let auth_response: AuthApiResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Error parsing auth response: {}", e)))?;

        if !auth_response.success {
            return Err(AppError::Unauthorized(
                auth_response.message.unwrap_or_else(|| "Autenticación fallida".to_string())
            ));
        }

        let data = auth_response.data
            .ok_or_else(|| AppError::ExternalApi("No data in auth response".to_string()))?;

        // Token expira en 24 horas
        let expires_at = Utc::now() + Duration::hours(24);

        Ok(AuthenticationResult {
            sso_token: data.sso_hopps,
            matricule_chauffeur: data.matricule_chauffeur,
            nom_chauffeur: data.nom_chauffeur,
            expires_at,
        })
    }

    pub async fn get_tournee(
        &self,
        sso_token: &str,
        matricule: &str,
        societe: &str,
        date: Option<&str>,
    ) -> Result<Vec<PackageData>, AppError> {
        let date_str = date
            .map(|d| d.to_string())
            .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());

        let tournee_request = TourneeApiRequest {
            code_societe: societe.to_string(),
            matricule: matricule.to_string(),
            date_heure_debut: format!("{}T00:00:00", date_str),
        };

        let response = self.client
            .post("https://wstournee-v2.colisprive.com/WS-TourneeColis/api/recupTourneeColisAvecNonProspect_POST/")
            .header("SsoHopps", sso_token)
            .header("Content-Type", "application/json")
            .json(&tournee_request)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Error en request de tournée: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!("Error obteniendo tournée: {}", response.status())));
        }

        let tournee_response: TourneeApiResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Error parsing tournee response: {}", e)))?;

        if !tournee_response.success {
            return Err(AppError::ExternalApi("Tournée request failed".to_string()));
        }

        let data = tournee_response.data
            .ok_or_else(|| AppError::ExternalApi("No data in tournee response".to_string()))?;

        // Convertir a PackageData
        let packages: Vec<colis_prive_dto::PackageData> = data.lst_lieu_article
            .into_iter()
            .map(|lieu| {
                let ref_colis = lieu.reference_colis.clone().unwrap_or_default();
                let nom = lieu.destinataire_nom.clone().unwrap_or_default();
                let addr1 = lieu.destinataire_adresse1.clone().unwrap_or_default();
                let ville = lieu.destinataire_ville.clone().unwrap_or_default();
                
                colis_prive_dto::PackageData {
                    // Campos principales
                    reference_colis: ref_colis.clone(),
                    destinataire_nom: nom.clone(),
                    destinataire_adresse1: lieu.destinataire_adresse1.clone(),
                    destinataire_adresse2: lieu.destinataire_adresse2.clone(),
                    destinataire_cp: lieu.destinataire_cp.clone(),
                    destinataire_ville: lieu.destinataire_ville.clone(),
                    coord_x_destinataire: lieu.coord_x_destinataire,
                    coord_y_destinataire: lieu.coord_y_destinataire,
                    statut: lieu.statut.clone(),
                    numero_ordre: lieu.numero_ordre,
                    
                    // Campos legacy
                    id: Some(ref_colis.clone()),
                    tracking_number: Some(ref_colis),
                    recipient_name: Some(nom),
                    address: Some(format!("{} {}", addr1, ville)),
                    status: lieu.statut.clone(),
                    instructions: None,
                    phone: None,
                    priority: None,
                    latitude: lieu.coord_y_destinataire,
                    longitude: lieu.coord_x_destinataire,
                    formatted_address: Some(format!("{} {}", addr1, ville)),
                    validation_method: None,
                    validation_confidence: None,
                    validation_warnings: None,
                    num_ordre_passage_prevu: lieu.numero_ordre,
                }
            })
            .collect();

        Ok(packages)
    }

    pub async fn optimize_tournee(
        &self,
        sso_token: &str,
        matricule: &str,
        societe: &str,
    ) -> Result<OptimizationResult, AppError> {
        let date_str = Utc::now().format("%Y-%m-%d").to_string();

        let optimize_request = serde_json::json!({
            "codeSociete": societe,
            "matricule": matricule,
            "dateHeureDebut": format!("{}T00:00:00", date_str),
            "coordX": null,
            "coordY": null,
        });

        log::info!("🚀 Enviando request de optimización a Colis Privé con token: {}...", &sso_token[..20]);
        log::info!("📋 Request data: {:?}", optimize_request);

        let response = self.client
            .post("https://wstournee-v2.colisprive.com/WS-TourneeColis/api/optimiserTourneeAvecValidation_POST/")
            .header("SsoHopps", sso_token)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/plain, */*")
            .header("Accept-Language", "fr-FR,fr;q=0.6")
            .header("Connection", "keep-alive")
            .header("Origin", "https://gestiontournee.colisprive.com")
            .header("Referer", "https://gestiontournee.colisprive.com/")
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Site", "same-site")
            .header("Sec-GPC", "1")
            .header("sec-ch-ua", r#""Brave";v="141", "Not?A_Brand";v="8", "Chromium";v="141""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""macOS""#)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36")
            .json(&optimize_request)
            .timeout(std::time::Duration::from_secs(90))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Error llamando API Colis Privé: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            log::error!("❌ Error en optimización: {} - {}", status, error_text);
            return Err(AppError::ExternalApi(format!("API Colis Privé retornó error: {}", status)));
        }

        let optimize_response: TourneeApiResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Error parsing optimize response: {}", e)))?;

        if !optimize_response.success {
            return Err(AppError::ExternalApi("Optimization request failed".to_string()));
        }

        let data = optimize_response.data
            .ok_or_else(|| AppError::ExternalApi("No data in optimization response".to_string()))?;

        // Convertir a PackageData
        let packages: Vec<colis_prive_dto::PackageData> = data.lst_lieu_article
            .into_iter()
            .map(|lieu| {
                let ref_colis = lieu.reference_colis.clone().unwrap_or_default();
                let nom = lieu.destinataire_nom.clone().unwrap_or_default();
                let addr1 = lieu.destinataire_adresse1.clone().unwrap_or_default();
                let ville = lieu.destinataire_ville.clone().unwrap_or_default();
                
                colis_prive_dto::PackageData {
                    // Campos principales
                    reference_colis: ref_colis.clone(),
                    destinataire_nom: nom.clone(),
                    destinataire_adresse1: lieu.destinataire_adresse1.clone(),
                    destinataire_adresse2: lieu.destinataire_adresse2.clone(),
                    destinataire_cp: lieu.destinataire_cp.clone(),
                    destinataire_ville: lieu.destinataire_ville.clone(),
                    coord_x_destinataire: lieu.coord_x_destinataire,
                    coord_y_destinataire: lieu.coord_y_destinataire,
                    statut: lieu.statut.clone(),
                    numero_ordre: lieu.numero_ordre,
                    
                    // Campos legacy
                    id: Some(ref_colis.clone()),
                    tracking_number: Some(ref_colis),
                    recipient_name: Some(nom),
                    address: Some(format!("{} {}", addr1, ville)),
                    status: lieu.statut.clone(),
                    instructions: None,
                    phone: None,
                    priority: None,
                    latitude: lieu.coord_y_destinataire,
                    longitude: lieu.coord_x_destinataire,
                    formatted_address: Some(format!("{} {}", addr1, ville)),
                    validation_method: None,
                    validation_confidence: None,
                    validation_warnings: None,
                    num_ordre_passage_prevu: lieu.numero_ordre,
                }
            })
            .collect();

        Ok(OptimizationResult {
            matricule_chauffeur: matricule.to_string(),
            date_tournee: date_str,
            packages,
        })
    }
}
