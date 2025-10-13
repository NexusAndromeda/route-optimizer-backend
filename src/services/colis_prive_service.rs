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
        let login_field = format!("{}_{}", societe, username.trim());
        
        let auth_payload = serde_json::json!({
            "login": login_field,
            "password": password,
            "societe": societe,
            "commun": {
                "dureeTokenInHour": 24
            }
        });

        let auth_url = format!("{}/api/auth/login/Membership", self.config.colis_prive_auth_url);
        
        log::info!("🔗 Usando curl para autenticación en {}...", auth_url);
        log::info!("🔑 Login field: {}", login_field);
        
        // Serializar payload
        let auth_payload_str = serde_json::to_string(&auth_payload)
            .map_err(|e| AppError::Internal(format!("Error serializando payload: {}", e)))?;

        log::info!("📦 Payload: {}", auth_payload_str);

        // Usar curl (más confiable que reqwest para Colis Privé)
        let curl_output = std::process::Command::new("curl")
            .arg("-X")
            .arg("POST")
            .arg(&auth_url)
            .arg("-H")
            .arg("Accept: application/json, text/plain, */*")
            .arg("-H")
            .arg("Accept-Language: fr-FR,fr;q=0.6")
            .arg("-H")
            .arg("Connection: keep-alive")
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-H")
            .arg("Origin: https://gestiontournee.colisprive.com")
            .arg("-H")
            .arg("Referer: https://gestiontournee.colisprive.com/")
            .arg("-H")
            .arg("Sec-Fetch-Dest: empty")
            .arg("-H")
            .arg("Sec-Fetch-Mode: cors")
            .arg("-H")
            .arg("Sec-Fetch-Site: same-site")
            .arg("-H")
            .arg("Sec-GPC: 1")
            .arg("-H")
            .arg("User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36")
            .arg("-H")
            .arg("sec-ch-ua: \"Chromium\";v=\"141\", \"Not=A?Brand\";v=\"24\", \"Brave\";v=\"141\"")
            .arg("-H")
            .arg("sec-ch-ua-mobile: ?0")
            .arg("-H")
            .arg("sec-ch-ua-platform: \"macOS\"")
            .arg("--data-raw")
            .arg(&auth_payload_str)
            .arg("--max-time")
            .arg("30")
            .arg("--silent")
            .arg("--show-error")
            .output()
            .map_err(|e| {
                log::error!("❌ Error ejecutando curl: {}", e);
                AppError::ExternalApi(format!("Error ejecutando curl: {}", e))
            })?;

        if !curl_output.status.success() {
            let error_msg = String::from_utf8_lossy(&curl_output.stderr);
            log::error!("❌ Curl falló: {}", error_msg);
            return Err(AppError::ExternalApi(format!("Curl falló: {}", error_msg)));
        }

        let response_body = String::from_utf8_lossy(&curl_output.stdout);
        log::info!("📥 Respuesta: {}", &response_body[..response_body.len().min(200)]);

        // Parsear la respuesta JSON
        let json_response: serde_json::Value = serde_json::from_str(&response_body)
            .map_err(|e| AppError::ExternalApi(format!("Error parsing auth response: {}", e)))?;

        // Extraer el SsoHopps
        let sso_token = json_response
            .get("SsoHopps")
            .or_else(|| json_response.get("ssoHopps"))
            .or_else(|| json_response.get("habilitacionAD")
                .and_then(|h| h.get("SsoHopps"))
                .and_then(|s| s.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|item| item.get("valeur")))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                log::error!("❌ Token no encontrado. Campos disponibles: {:?}", 
                    json_response.as_object().map(|obj| obj.keys().collect::<Vec<_>>()));
                AppError::ExternalApi("No SsoHopps in response".to_string())
            })?
            .to_string();

        let matricule_chauffeur = json_response
            .get("matriculeChauffeur")
            .and_then(|v| v.as_str())
            .unwrap_or(username)
            .to_string();

        let nom_chauffeur = json_response
            .get("nomChauffeur")
            .and_then(|v| v.as_str())
            .unwrap_or("Chauffeur")
            .to_string();

        log::info!("✅ Autenticación exitosa - Token obtenido");

        // Token expira en 24 horas
        let expires_at = Utc::now() + Duration::hours(24);

        Ok(AuthenticationResult {
            sso_token,
            matricule_chauffeur,
            nom_chauffeur,
            expires_at,
        })
    }

    pub async fn get_tournee(
        &self,
        sso_token: &str,
        matricule: &str,
        societe: &str,
        date: Option<&str>,
    ) -> Result<Vec<colis_prive_dto::PackageData>, AppError> {
        let date_str = date
            .map(|d| d.to_string())
            .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());

        let matricule_completo = format!("{}_{}", societe, matricule);
        
        let payload = serde_json::json!({
            "Matricule": matricule_completo,
            "DateDebut": date_str
        });

        let payload_str = serde_json::to_string(&payload)
            .map_err(|e| AppError::Internal(format!("Error serializando payload: {}", e)))?;

        let tournee_url = format!("{}/WS-TourneeColis/api/getTourneeByMatriculeDistributeurDateDebut_POST", self.config.colis_prive_tournee_url);
        
        log::info!("📤 Llamando a tournée: {}", tournee_url);
        log::info!("📦 Payload: {}", payload_str);
        log::info!("🔑 Token: {}...", &sso_token[..20.min(sso_token.len())]);

        // Usar curl
        let curl_output = std::process::Command::new("curl")
            .arg("-X")
            .arg("POST")
            .arg(&tournee_url)
            .arg("-H")
            .arg("Accept: application/json, text/plain, */*")
            .arg("-H")
            .arg("Accept-Language: fr-FR,fr;q=0.6")
            .arg("-H")
            .arg("Connection: keep-alive")
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-H")
            .arg("Origin: https://gestiontournee.colisprive.com")
            .arg("-H")
            .arg("Referer: https://gestiontournee.colisprive.com/")
            .arg("-H")
            .arg("Sec-Fetch-Dest: empty")
            .arg("-H")
            .arg("Sec-Fetch-Mode: cors")
            .arg("-H")
            .arg("Sec-Fetch-Site: same-site")
            .arg("-H")
            .arg("Sec-GPC: 1")
            .arg("-H")
            .arg(format!("SsoHopps: {}", sso_token))
            .arg("-H")
            .arg("User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36")
            .arg("-H")
            .arg("sec-ch-ua: \"Chromium\";v=\"141\", \"Not=A?Brand\";v=\"24\", \"Brave\";v=\"141\"")
            .arg("-H")
            .arg("sec-ch-ua-mobile: ?0")
            .arg("-H")
            .arg("sec-ch-ua-platform: \"macOS\"")
            .arg("--data-raw")
            .arg(&payload_str)
            .arg("--max-time")
            .arg("30")
            .arg("--silent")
            .arg("--show-error")
            .output()
            .map_err(|e| AppError::ExternalApi(format!("Error ejecutando curl: {}", e)))?;

        if !curl_output.status.success() {
            let error_msg = String::from_utf8_lossy(&curl_output.stderr);
            log::error!("❌ Curl falló: {}", error_msg);
            return Err(AppError::ExternalApi(format!("Curl falló: {}", error_msg)));
        }

        let response_str = String::from_utf8_lossy(&curl_output.stdout);
        log::info!("📥 Respuesta recibida: {} bytes", response_str.len());

        // Parsear la respuesta JSON
        let tournee_data: serde_json::Value = serde_json::from_str(&response_str)
            .map_err(|e| AppError::ExternalApi(format!("Error parsing tournee response: {}", e)))?;

        // Extraer paquetes de LstLieuArticle
        let lst_lieu_article = tournee_data
            .get("LstLieuArticle")
            .and_then(|v| v.as_array())
            .ok_or_else(|| AppError::ExternalApi("No LstLieuArticle in response".to_string()))?;

        // Convertir a PackageData
        let packages: Vec<colis_prive_dto::PackageData> = lst_lieu_article
            .iter()
            .filter_map(|package| {
                // Filtrar solo COLIS
                let metier = package.get("metier")?.as_str().unwrap_or("UNKNOWN");
                if metier != "COLIS" {
                    return None;
                }
                
                let ref_colis = package.get("refExterneArticle")?.as_str()?.to_string();
                let nom = package.get("nomDestinataire")?.as_str()?.to_string();
                let addr1 = package.get("LibelleVoieOrigineDestinataire")?.as_str()?.to_string();
                let cp = package.get("codePostalOrigineDestinataire")?.as_str()?.to_string();
                let ville = package.get("LibelleLocaliteOrigineDestinataire")?.as_str()?.to_string();
                
                Some(colis_prive_dto::PackageData {
                    // Campos principales
                    reference_colis: ref_colis.clone(),
                    destinataire_nom: nom.clone(),
                    destinataire_adresse1: Some(addr1.clone()),
                    destinataire_adresse2: None,
                    destinataire_cp: Some(cp.clone()),
                    destinataire_ville: Some(ville.clone()),
                    coord_x_destinataire: package.get("coordXOrigineDestinataire").and_then(|v| v.as_f64()),
                    coord_y_destinataire: package.get("coordYOrigineDestinataire").and_then(|v| v.as_f64()),
                    statut: package.get("codeStatutArticle").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    numero_ordre: package.get("numeroOrdre").and_then(|v| v.as_i64()).map(|n| n as i32),
                    
                    // Campos legacy
                    id: Some(package.get("idArticle")?.as_str()?.to_string()),
                    tracking_number: Some(ref_colis.clone()),
                    recipient_name: Some(nom.clone()),
                    address: Some(format!("{}, {} {}", addr1, cp, ville)),
                    status: package.get("codeStatutArticle").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    instructions: package.get("PreferenceLivraison").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    phone: package.get("telephoneMobileDestinataire").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    priority: None,
                    latitude: package.get("coordYOrigineDestinataire").and_then(|v| v.as_f64()),
                    longitude: package.get("coordXOrigineDestinataire").and_then(|v| v.as_f64()),
                    formatted_address: Some(format!("{}, {} {}", addr1, cp, ville)),
                    validation_method: None,
                    validation_confidence: None,
                    validation_warnings: None,
                    num_ordre_passage_prevu: package.get("numeroOrdre").and_then(|v| v.as_i64()).map(|n| n as i32),
                })
            })
            .collect();

        log::info!("✅ Paquetes obtenidos: {}", packages.len());

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
