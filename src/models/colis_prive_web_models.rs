use serde::{Deserialize, Serialize};

// 🎯 MODELOS WEB ESENCIALES PARA COLIS PRIVÉ
// Solo los modelos necesarios para la API web

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
    pub societe: String,
    pub commun: Commun,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commun {
    pub duree_token_in_hour: i32,
}

// Modelo para autenticación web
#[derive(Debug, Serialize, Deserialize)]
pub struct ColisPriveWebAuthRequest {
    pub login: String,
    pub password: String,
    pub societe: String,
    pub commun: Commun,
}

// Modelo para respuesta de autenticación web
#[derive(Debug, Serialize, Deserialize)]
pub struct ColisPriveWebAuthResponse {
    pub is_authentif: bool,
    pub identity: String,
    pub matricule: String,
    pub societe: String,
    pub tokens: ColisPriveTokens,
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColisPriveTokens {
    pub sso_hopps: String,
}

// Modelo para request de tournée web
#[derive(Debug, Serialize, Deserialize)]
pub struct ColisPriveWebTourneeRequest {
    pub societe: String,
    pub matricule: String,
    pub date_debut: String,
    pub agence: Option<String>,
    pub concentrateur: Option<String>,
}

// Modelo para respuesta de tournée web
#[derive(Debug, Serialize, Deserialize)]
pub struct ColisPriveWebTourneeResponse {
    pub statut: String,
    pub date: String,
    pub bean_today: BeanToday,
    pub list_bean_distributeur: Vec<BeanDistributeur>,
    pub list_bean_localite: Vec<BeanLocalite>,
    pub list_bean_tournee: Vec<BeanTournee>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BeanToday {
    pub date: String,
    pub nb_colis: i32,
    pub nb_colis_collecte: i32,
    pub nb_colis_premium: i32,
    pub nb_non_attribue: i32,
    pub nb_collecte_non_attribue: i32,
    pub nb_distribue: i32,
    pub nb_non_attribue_premium: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BeanDistributeur {
    pub matricule_distributeur: String,
    pub nom_distributeur: String,
    pub is_colis_affecte: bool,
    pub duree_journee_in_minute: i32,
    pub nb_pause_en_minutes: i32,
    pub date_debut_tournee: String,
    pub date_debut_pause: String,
    pub date_fin_pause: String,
    pub nb_colis_max_by_day: i32,
    pub bean_alerte: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BeanLocalite {
    pub code_postal: String,
    pub libelle_localite: String,
    pub nb_colis: i32,
    pub is_has_colis: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BeanTournee {
    pub code_tournee: String,
    pub code_tournee_mcp: String,
    pub statut_tournee: String,
    pub list_bean_localite: Vec<BeanLocalite>,
    pub nb_colis: i32,
    pub nb_colis_acollecter: i32,
    pub nb_colis_collecte: i32,
    pub nb_colis_premium: i32,
    pub nb_colis_restant_premium_adistribue: i32,
    pub bean_distributeur: BeanDistributeur,
    pub nb_colis_distribue: i32,
    pub nb_colis_restant_adistribue: i32,
    pub nb_colis_traite: i32,
    pub nb_colis_traite_premium: i32,
    pub duree_tournee_prevu_in_minute: i32,
    pub duree_tournee_realise_in_minute: i32,
    pub duree_tournee_restante_minutes: i32,
    pub nb_colis_relais: i32,
    pub nb_colis_relais_premium: i32,
    pub nb_colis_casier: i32,
    pub nb_colis_casier_premium: i32,
    pub alerte_tournee_preparation: Option<serde_json::Value>,
    pub alerte_tournee_distribution: Option<serde_json::Value>,
    pub code_centre: String,
    pub code_point_concentration: String,
}
