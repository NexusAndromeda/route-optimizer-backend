use reqwest::Client;
use serde::{Deserialize, Serialize};
use log;

// Estructura de la compañía tal como viene de la API de Colis Privé
#[derive(Debug, Deserialize)]
pub struct ColisPriveCompanyRawValue {
    #[serde(rename = "CLI_ID_CLIENT")]
    pub cli_id_client: i32,
    #[serde(rename = "CLI_LI_CLIENT")]
    pub cli_li_client: String,
    #[serde(rename = "CLI_LI_CLIENT_COURT")]
    pub cli_li_client_court: String,
    #[serde(rename = "CLI_NO_CLIENT_CRM")]
    pub cli_no_client_crm: String,
    #[serde(rename = "CLI_TYPE")]
    pub cli_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ColisPriveCompanyRaw {
    #[serde(rename = "Key")]
    pub key: i32,
    #[serde(rename = "Value")]
    pub value: ColisPriveCompanyRawValue,
}

// Estructura de la respuesta de la API de Colis Privé
#[derive(Debug, Deserialize)]
pub struct ColisPriveCompaniesResponseRaw {
    #[serde(rename = "LCli")]
    pub l_cli: Vec<ColisPriveCompanyRaw>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColisPriveCompany {
    pub libelle: String,
    pub code: String,
}

pub struct ColisPriveCompaniesService {
    client: Client,
    base_url: String,
}

impl ColisPriveCompaniesService {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn get_companies(&self) -> Result<Vec<ColisPriveCompany>, Box<dyn std::error::Error>> {
        let url = format!("{}/REST/ClientExtranetlightByTypeClient?TypeClient=PRESTATAIRECOLIS", self.base_url);
        
        log::info!("🏢 Llamando a Colis Privé (API real): {}", url);
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/json, text/plain, */*")
            .header("Accept-Language", "fr-FR,fr;q=0.5")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .header("Origin", "https://gestiontournee.colisprive.com")
            .header("Pragma", "no-cache")
            .header("Referer", "https://gestiontournee.colisprive.com/")
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36")
            .header("sec-ch-ua", r#""Chromium";v="140", "Not=A?Brand";v="24", "Brave";v="140""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""macOS""#)
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Site", "same-site")
            .header("Sec-GPC", "1")
            .send()
            .await?;
        
        log::info!("📡 Respuesta recibida: {}", response.status());
        
        let raw_response: ColisPriveCompaniesResponseRaw = response.json().await?;
        
        log::info!("📊 Empresas recibidas: {}", raw_response.l_cli.len());
        
        // Mapear a nuestro formato
        let companies: Vec<ColisPriveCompany> = raw_response
            .l_cli
            .into_iter()
            .map(|raw| ColisPriveCompany {
                libelle: raw.value.cli_li_client,
                code: raw.value.cli_no_client_crm,
            })
            .collect();
        
        log::info!("✅ Empresas mapeadas: {}", companies.len());
        
        Ok(companies)
    }
}

// Función helper pública para obtener empresas
pub async fn fetch_all_companies() -> Result<Vec<crate::models::colis_prive_company::ColisPriveCompany>, crate::utils::errors::AppError> {
    let service = ColisPriveCompaniesService::new(
        std::env::var("COLIS_PRIVE_REFERENTIEL_URL")
            .unwrap_or_else(|_| "https://wsreferentiel-v2.colisprive.com/WS_RefDistributeur/RefDistributeurConsolideExtranetToExterne.svc".to_string())
    );
    
    service.get_companies()
        .await
        .map(|companies| {
            companies.into_iter().map(|c| crate::models::colis_prive_company::ColisPriveCompany {
                code: c.code,
                name: c.libelle,
                description: None,
            }).collect()
        })
        .map_err(|e| crate::utils::errors::AppError::ExternalApi(format!("Error fetching companies: {}", e)))
}

