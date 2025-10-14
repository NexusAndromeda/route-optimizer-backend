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

// Estructura específica para la respuesta de optimización
#[derive(Debug, Deserialize)]
struct OptimizationApiResponse {
    #[serde(rename = "MatriculeChauffeur")]
    matricule_chauffeur: String,
    #[serde(rename = "DateTournee")]
    date_tournee: String,
    #[serde(rename = "LstLieuArticle")]
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

        // Extraer el token - está en tokens.SsoHopps (el largo)
        let sso_token = json_response
            .get("tokens")
            .and_then(|t| t.get("SsoHopps"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                log::error!("❌ Token no encontrado en tokens.SsoHopps");
                log::error!("🔍 Response keys: {:?}", 
                    json_response.as_object().map(|obj| obj.keys().collect::<Vec<_>>()));
                AppError::ExternalApi("Token no encontrado en response".to_string())
            })?
            .to_string();
        
        log::info!("✅ Token extraído ({}... bytes)", sso_token.len());

        let matricule_chauffeur = json_response
            .get("matricule")
            .or_else(|| json_response.get("matriculeChauffeur"))
            .and_then(|v| v.as_str())
            .unwrap_or(&login_field)
            .to_string();

        let nom_chauffeur = json_response
            .get("nom")
            .or_else(|| json_response.get("nomChauffeur"))
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
        let now = Utc::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let datetime_iso = now.to_rfc3339();

        // Construir matricule completo (SOCIETE_MATRICULE)
        let full_matricule = if matricule.contains('_') {
            matricule.to_string()
        } else {
            format!("{}_{}", societe, matricule)
        };

        // Usar exactamente el mismo formato que la página oficial
        let optimize_request = serde_json::json!({
            "CodeSociete": societe,
            "Matricule": full_matricule,
            "DateHeureDebut": datetime_iso,
            "CoordX": null,
            "CoordY": null,
            "CoordRetourX": null,
            "CoordRetourY": null,
            "CodeTournee": format!("{}-{}", full_matricule, now.format("%Y%m%d")),
            "IsModeOptimToutCPConfondus": false,
            "PauseHeureDebut": null,
            "PauseDuree": null
        });

        let optimize_payload = serde_json::to_string(&optimize_request)
            .map_err(|e| AppError::ExternalApi(format!("Error serializing optimize request: {}", e)))?;

        log::info!("🚀 Enviando request de optimización a Colis Privé con token: {}...", &sso_token[..20.min(sso_token.len())]);
        log::info!("📋 Request data: {}", optimize_payload);

        let optimize_url = "https://wstournee-v2.colisprive.com/WS-TourneeColis/api/optimiserTourneeAvecValidation_POST/";

        // Usar curl (más confiable que reqwest para Colis Privé)
        let curl_output = std::process::Command::new("curl")
            .arg("-X")
            .arg("POST")
            .arg(optimize_url)
            .arg("-H")
            .arg("Accept: application/json, text/plain, */*")
            .arg("-H")
            .arg("Accept-Language: fr-FR,fr;q=0.6")
            .arg("-H")
            .arg("Connection: keep-alive")
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-H")
            .arg(format!("SsoHopps: {}", sso_token))
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
            .arg(&optimize_payload)
            .arg("--max-time")
            .arg("90")
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
        log::info!("📥 Respuesta optimización recibida: {} bytes", response_body.len());

        // Primero intentar parsear como JSON genérico para detectar errores
        let json_value: serde_json::Value = serde_json::from_str(&response_body)
            .map_err(|e| {
                log::error!("❌ Error parsing JSON response: {}", e);
                log::error!("📄 Response body: {}", &response_body[..response_body.len().min(500)]);
                AppError::ExternalApi(format!("Error parsing JSON response: {}", e))
            })?;

        // Verificar si hay un mensaje de error
        if let Some(error_msg) = json_value.get("Message").and_then(|m| m.as_str()) {
            log::error!("❌ Error de Colis Privé: {}", error_msg);
            return Err(AppError::ExternalApi(format!("Colis Privé error: {}", error_msg)));
        }

        // Intentar parsear como respuesta de optimización (estructura diferente)
        let optimize_response: OptimizationApiResponse = serde_json::from_value(json_value)
            .map_err(|e| {
                log::error!("❌ Error parsing optimize response: {}", e);
                log::error!("📄 Response body: {}", &response_body[..response_body.len().min(500)]);
                AppError::ExternalApi(format!("Error parsing optimize response: {}", e))
            })?;

        log::info!("✅ Optimización exitosa para: {}", optimize_response.matricule_chauffeur);

        // Convertir a PackageData
        let packages: Vec<colis_prive_dto::PackageData> = optimize_response.lst_lieu_article
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
            matricule_chauffeur: optimize_response.matricule_chauffeur,
            date_tournee: optimize_response.date_tournee,
            packages,
        })
    }
}
