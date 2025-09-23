//! Cliente HTTP para Colis Privé (API Web)
//! 
//! Este módulo contiene el cliente HTTP para la API web de Colis Privé,
//! incluyendo tanto el API básico como el API detalle.

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cliente HTTP para Colis Privé (API Web + API Detalle)
pub struct ColisPriveWebClient {
    pub client: Client,
    pub auth_base_url: String,
    pub tournee_base_url: String,
    pub detail_base_url: String,
}

/// Respuesta del API detalle de Colis Privé
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColisDetailResponse {
    pub success: bool,
    pub data: Option<ColisDetailData>,
    pub message: Option<String>,
}

/// Datos detallados del paquete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColisDetailData {
    pub ref_colis: String,
    pub code_barre_complet: Option<String>,
    pub adresse_complete: Option<String>,
    pub code_postal: Option<String>,
    pub ville: Option<String>,
    pub pays: Option<String>,
    pub coordonnees: Option<Coordonnees>,
    pub donnees_physiques: Option<DonneesPhysiques>,
    pub historique: Option<Vec<HistoriqueItem>>,
    pub commentaires: Option<String>,
    pub instructions_livraison: Option<String>,
    pub horaires_livraison: Option<HorairesLivraison>,
    pub contact: Option<Contact>,
}

/// Coordenadas geográficas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordonnees {
    pub latitude: f64,
    pub longitude: f64,
    pub precision: Option<String>,
}

/// Datos físicos del paquete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonneesPhysiques {
    pub poids: Option<f64>,
    pub unite_poids: Option<String>,
    pub dimensions: Option<Dimensions>,
    pub valeur: Option<f64>,
    pub devise: Option<String>,
}

/// Dimensiones del paquete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimensions {
    pub longueur: Option<f64>,
    pub largeur: Option<f64>,
    pub hauteur: Option<f64>,
    pub unite: Option<String>,
}

/// Elemento del historial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoriqueItem {
    pub date: String,
    pub heure: String,
    pub statut: String,
    pub description: Option<String>,
    pub lieu: Option<String>,
}

/// Horarios de entrega
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorairesLivraison {
    pub debut: Option<String>,
    pub fin: Option<String>,
    pub jours_semaine: Option<Vec<String>>,
}

/// Información de contacto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub nom: Option<String>,
    pub prenom: Option<String>,
    pub telephone: Option<String>,
    pub email: Option<String>,
}

/// Configuración del cache para API detalle
#[derive(Debug, Clone)]
pub struct DetailCacheConfig {
    pub ttl_seconds: u64,
    pub max_entries: usize,
}

impl Default for DetailCacheConfig {
    fn default() -> Self {
        Self {
            ttl_seconds: 3600, // 1 hora
            max_entries: 1000,
        }
    }
}

impl ColisPriveWebClient {
    /// Crear nuevo cliente HTTP para API web con URLs configurables
    pub fn new(
        auth_base_url: String,
        tournee_base_url: String,
        detail_base_url: String,
    ) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            auth_base_url,
            tournee_base_url,
            detail_base_url,
        })
    }

    /// Obtener datos detallados de un paquete específico
    pub async fn get_package_detail(
        &self,
        ref_colis: &str,
        sso_token: &str,
    ) -> Result<ColisDetailResponse> {
        let url = format!(
            "{}/WS-TourneeColis/api/GetBeanSuiviColisByRefColisWithTracabilite/{}",
            self.detail_base_url,
            ref_colis
        );

        let response = self
            .client
            .post(&url)
            .header("Accept", "application/json, text/plain, */*")
            .header("Accept-Language", "fr-FR,fr;q=0.5")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .header("Content-Length", "0")
            .header("Origin", "https://gestiontournee.colisprive.com")
            .header("Pragma", "no-cache")
            .header("Referer", "https://gestiontournee.colisprive.com/")
            .header("SsoHopps", sso_token)
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

        if !response.status().is_success() {
            return Ok(ColisDetailResponse {
                success: false,
                data: None,
                message: Some(format!("Error HTTP: {}", response.status())),
            });
        }

        let detail_response: ColisDetailResponse = response.json().await?;
        Ok(detail_response)
    }

    /// Obtener múltiples paquetes detallados en lote
    pub async fn get_packages_detail_batch(
        &self,
        ref_colis_list: &[String],
        sso_token: &str,
    ) -> Result<HashMap<String, ColisDetailResponse>> {
        let mut results = HashMap::new();
        
        // Procesar en lotes de 5 para evitar sobrecarga (reducido por el POST)
        for chunk in ref_colis_list.chunks(5) {
            let mut futures = Vec::new();
            
            for ref_colis in chunk {
                let client = &self.client;
                let url = format!(
                    "{}/WS-TourneeColis/api/GetBeanSuiviColisByRefColisWithTracabilite/{}",
                    self.detail_base_url,
                    ref_colis
                );
                let token = sso_token.to_string();
                let ref_colis_clone = ref_colis.clone();
                
                let future = async move {
                    let response = client
                        .post(&url)
                        .header("Accept", "application/json, text/plain, */*")
                        .header("Accept-Language", "fr-FR,fr;q=0.5")
                        .header("Cache-Control", "no-cache")
                        .header("Connection", "keep-alive")
                        .header("Content-Length", "0")
                        .header("Origin", "https://gestiontournee.colisprive.com")
                        .header("Pragma", "no-cache")
                        .header("Referer", "https://gestiontournee.colisprive.com/")
                        .header("SsoHopps", &token)
                        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36")
                        .header("sec-ch-ua", r#""Chromium";v="140", "Not=A?Brand";v="24", "Brave";v="140""#)
                        .header("sec-ch-ua-mobile", "?0")
                        .header("sec-ch-ua-platform", r#""macOS""#)
                        .header("Sec-Fetch-Dest", "empty")
                        .header("Sec-Fetch-Mode", "cors")
                        .header("Sec-Fetch-Site", "same-site")
                        .header("Sec-GPC", "1")
                        .send()
                        .await;
                    
                    match response {
                        Ok(resp) => {
                            if resp.status().is_success() {
                                match resp.json::<ColisDetailResponse>().await {
                                    Ok(detail) => (ref_colis_clone, detail),
                                    Err(e) => (ref_colis_clone, ColisDetailResponse {
                                        success: false,
                                        data: None,
                                        message: Some(format!("Error parsing JSON: {}", e)),
                                    }),
                                }
                            } else {
                                (ref_colis_clone, ColisDetailResponse {
                                    success: false,
                                    data: None,
                                    message: Some(format!("Error HTTP: {}", resp.status())),
                                })
                            }
                        }
                        Err(e) => (ref_colis_clone, ColisDetailResponse {
                            success: false,
                            data: None,
                            message: Some(format!("Error de red: {}", e)),
                        }),
                    }
                };
                
                futures.push(future);
            }
            
            // Esperar a que terminen todos los requests del lote
            let batch_results = futures::future::join_all(futures).await;
            for (ref_colis, response) in batch_results {
                results.insert(ref_colis, response);
            }
            
            // Pausa más larga entre lotes para evitar rate limiting con POST
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
        
        Ok(results)
    }
}
