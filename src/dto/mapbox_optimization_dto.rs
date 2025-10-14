//! DTOs para Mapbox Optimization API v2
//! 
//! Este módulo define las estructuras de datos para interactuar con
//! la API de optimización de rutas de Mapbox.

use serde::{Deserialize, Serialize};

/// Request para enviar a Mapbox Optimization API
#[derive(Debug, Serialize)]
pub struct MapboxOptimizationRequest {
    pub version: u32,
    pub locations: Vec<MapboxLocation>,
    pub vehicles: Vec<MapboxVehicle>,
    pub services: Vec<MapboxService>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<MapboxOptions>,
}

/// Ubicación física en el espacio
#[derive(Debug, Serialize, Clone)]
pub struct MapboxLocation {
    pub name: String,
    pub coordinates: [f64; 2], // [longitude, latitude]
}

/// Vehículo en la flota
#[derive(Debug, Serialize)]
pub struct MapboxVehicle {
    pub name: String,
    pub start_location: String,
    pub end_location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<Vec<i32>>,
}

/// Servicio a realizar en una ubicación
#[derive(Debug, Serialize)]
pub struct MapboxService {
    pub name: String,
    pub location: String,
    pub duration: u32, // duración en segundos
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<Vec<i32>>,
}

/// Opciones de optimización
#[derive(Debug, Serialize)]
pub struct MapboxOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objectives: Option<Vec<String>>,
}

/// Response de Mapbox Optimization API v1
#[derive(Debug, Deserialize)]
pub struct MapboxOptimizationResponse {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waypoints: Option<Vec<MapboxWaypointV1>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trips: Option<Vec<MapboxTripV1>>,
}

/// Waypoint en la respuesta v1
#[derive(Debug, Deserialize)]
pub struct MapboxWaypointV1 {
    pub waypoint_index: usize,
    pub trips_index: usize,
    pub name: String,
    pub location: Vec<f64>,
}

/// Trip en la respuesta v1
#[derive(Debug, Deserialize)]
pub struct MapboxTripV1 {
    pub geometry: serde_json::Value,
    pub legs: Vec<serde_json::Value>,
    pub weight_name: String,
    pub weight: f64,
    pub duration: f64,
    pub distance: f64,
}

/// Response de Mapbox Optimization API v2 (Beta)
#[derive(Debug, Deserialize)]
pub struct MapboxOptimizationV2Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dropped: Option<MapboxDropped>,
    pub routes: Vec<MapboxRoute>,
}

/// Elementos que no se pudieron incluir en la solución
#[derive(Debug, Deserialize)]
pub struct MapboxDropped {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub services: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipments: Option<Vec<String>>,
}

/// Ruta optimizada para un vehículo
#[derive(Debug, Deserialize)]
pub struct MapboxRoute {
    pub vehicle: String,
    pub stops: Vec<MapboxStop>,
}

/// Parada en la ruta
#[derive(Debug, Deserialize)]
pub struct MapboxStop {
    #[serde(rename = "type")]
    pub stop_type: String,
    pub location: String,
    pub eta: String, // ISO 8601 timestamp
    pub odometer: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub services: Option<Vec<String>>,
}

/// Response de submit (cuando se envía el problema)
#[derive(Debug, Deserialize)]
pub struct MapboxSubmitResponse {
    pub id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_date: Option<String>,
}

/// Request para nuestro endpoint interno (adaptado desde Colis Privé)
#[derive(Debug, Deserialize)]
pub struct OptimizationRequest {
    pub matricule: String,
    pub societe: String,
    pub packages: Vec<OptimizationPackage>,
}

/// Paquete para optimización
#[derive(Debug, Deserialize, Clone)]
pub struct OptimizationPackage {
    pub id: String,
    pub reference_colis: String,
    pub destinataire_nom: String,
    pub destinataire_adresse1: Option<String>,
    pub destinataire_cp: Option<String>,
    pub destinataire_ville: Option<String>,
    pub coord_x_destinataire: Option<f64>,
    pub coord_y_destinataire: Option<f64>,
    pub statut: Option<String>,
}

/// Response de nuestro endpoint interno (compatible con frontend)
#[derive(Debug, Serialize)]
pub struct OptimizationResponse {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<OptimizationData>,
}

/// Datos de optimización (compatible con frontend)
#[derive(Debug, Serialize)]
pub struct OptimizationData {
    pub matricule_chauffeur: Option<String>,
    pub date_tournee: Option<String>,
    pub optimized_packages: Vec<OptimizedPackage>,
}

/// Paquete optimizado (compatible con frontend)
#[derive(Debug, Serialize, Clone)]
pub struct OptimizedPackage {
    pub id: Option<String>,
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
    pub tracking_number: Option<String>,
    pub recipient_name: Option<String>,
    pub address: Option<String>,
    pub status: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub formatted_address: Option<String>,
    pub num_ordre_passage_prevu: Option<i32>,
    pub eta: Option<String>, // Nueva: tiempo estimado de llegada
}

impl From<OptimizationPackage> for OptimizedPackage {
    fn from(pkg: OptimizationPackage) -> Self {
        let address = if let (Some(addr1), Some(cp), Some(ville)) = 
            (&pkg.destinataire_adresse1, &pkg.destinataire_cp, &pkg.destinataire_ville) {
            Some(format!("{}, {}, {}", addr1, cp, ville))
        } else {
            None
        };

        Self {
            id: Some(pkg.id.clone()),
            reference_colis: pkg.reference_colis.clone(),
            destinataire_nom: pkg.destinataire_nom.clone(),
            destinataire_adresse1: pkg.destinataire_adresse1.clone(),
            destinataire_adresse2: None,
            destinataire_cp: pkg.destinataire_cp.clone(),
            destinataire_ville: pkg.destinataire_ville.clone(),
            coord_x_destinataire: pkg.coord_x_destinataire,
            coord_y_destinataire: pkg.coord_y_destinataire,
            statut: pkg.statut.clone(),
            numero_ordre: None, // Se asignará después de la optimización
            tracking_number: Some(pkg.reference_colis.clone()),
            recipient_name: Some(pkg.destinataire_nom.clone()),
            address: address.clone(),
            status: pkg.statut.clone(),
            latitude: pkg.coord_y_destinataire,
            longitude: pkg.coord_x_destinataire,
            formatted_address: address,
            num_ordre_passage_prevu: None, // Se asignará después de la optimización
            eta: None, // Se asignará después de la optimización
        }
    }
}
