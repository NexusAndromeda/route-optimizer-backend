use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Roles del sistema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserRole {
    Livreur,
    Admin,
    SuperAdmin,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Livreur => "livreur",
            UserRole::Admin => "admin",
            UserRole::SuperAdmin => "super_admin",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "livreur" => Some(UserRole::Livreur),
            "admin" => Some(UserRole::Admin),
            "super_admin" => Some(UserRole::SuperAdmin),
            _ => None,
        }
    }
}

/// Información del usuario autenticado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub role: UserRole,
    pub company_id: Option<String>,
    pub tournee_id: Option<String>, // Solo para livreurs
    pub permissions: Vec<String>,
}

/// Claims del JWT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String, // user_id
    pub username: String,
    pub role: String,
    pub company_id: Option<String>,
    pub tournee_id: Option<String>,
    pub permissions: Vec<String>,
    pub exp: i64, // expiration timestamp
    pub iat: i64, // issued at timestamp
}

/// Request de login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub user_type: UserType, // livreur o admin
}

/// Tipo de usuario para el login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserType {
    Livreur,
    Admin,
}

/// Response de login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub token: Option<String>,
    pub user_info: Option<UserInfo>,
    pub message: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Request de refresh token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub token: String,
}

/// Response de refresh token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub success: bool,
    pub token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub message: Option<String>,
}

/// Información de la sesión activa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub user_id: String,
    pub username: String,
    pub role: UserRole,
    pub company_id: Option<String>,
    pub tournee_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub is_active: bool,
}

/// Cache de tokens Ssshops
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsshopsTokenCache {
    pub company_id: String,
    pub driver_matricule: String,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_valid: bool,
}

/// Cache de tournée
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourneeCache {
    pub tournee_id: String, // username del driver (A187518)
    pub driver_matricule: String,
    pub company_id: String,
    pub status: String, // active, completed, paused
    pub packages: TourneePackages,
    pub optimization: Option<TourneeOptimization>,
    pub last_activity: DateTime<Utc>,
    pub version: u32,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourneePackages {
    pub singles: Vec<serde_json::Value>,
    pub groups: Vec<serde_json::Value>,
    pub problematic: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourneeOptimization {
    pub optimized: bool,
    pub order: Vec<usize>,
    pub timestamp: DateTime<Utc>,
    pub total_distance: Option<f64>,
    pub total_duration: Option<f64>,
}

/// Request de sincronización de tournée
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourneeSyncRequest {
    pub tournee_id: String,
    pub packages: TourneePackages,
    pub optimization: Option<TourneeOptimization>,
    pub version: u32,
    pub checksum: String,
}

/// Response de sincronización de tournée
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourneeSyncResponse {
    pub success: bool,
    pub tournee: Option<TourneeCache>,
    pub message: Option<String>,
    pub conflicts: Option<Vec<String>>,
}

/// Lista de tournées para admin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourneeListResponse {
    pub success: bool,
    pub tournees: Vec<TourneeSummary>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourneeSummary {
    pub tournee_id: String,
    pub driver_name: String,
    pub driver_matricule: String,
    pub company_id: String,
    pub status: String,
    pub total_packages: usize,
    pub delivered_packages: usize,
    pub problematic_packages: usize,
    pub last_activity: DateTime<Utc>,
    pub is_optimized: bool,
}
