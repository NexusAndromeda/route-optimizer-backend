//! Configuración de variables de entorno
//! 
//! Este módulo maneja la configuración del entorno y variables de configuración.

use std::env;

/// Configuración del entorno
#[derive(Debug, Clone)]
pub struct EnvironmentConfig {
    pub environment: String,
    pub port: u16,
    pub host: String,
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub cors_origins: Vec<String>,
    pub rate_limit_requests: u32,
    pub rate_limit_window: u64,
    pub mapbox_token: Option<String>,
    // URLs de Colis Privé
    pub colis_prive_auth_url: String,
    pub colis_prive_tournee_url: String,
    pub colis_prive_detail_url: String,
    pub colis_prive_gestion_url: String,
    pub colis_prive_referentiel_url: String,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            environment: env::var("ENVIRONMENT").expect("ENVIRONMENT must be set"),
            port: env::var("PORT")
                .expect("PORT must be set")
                .parse()
                .expect("PORT must be a valid number"),
            host: env::var("HOST").expect("HOST must be set"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            jwt_expiration: env::var("JWT_EXPIRATION")
                .expect("JWT_EXPIRATION must be set")
                .parse()
                .expect("JWT_EXPIRATION must be a valid number"),
            cors_origins: env::var("CORS_ORIGINS")
                .expect("CORS_ORIGINS must be set")
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            rate_limit_requests: env::var("RATE_LIMIT_REQUESTS")
                .expect("RATE_LIMIT_REQUESTS must be set")
                .parse()
                .expect("RATE_LIMIT_REQUESTS must be a valid number"),
            rate_limit_window: env::var("RATE_LIMIT_WINDOW")
                .expect("RATE_LIMIT_WINDOW must be set")
                .parse()
                .expect("RATE_LIMIT_WINDOW must be a valid number"),
            mapbox_token: env::var("MAPBOX_TOKEN").ok(),
            // URLs de Colis Privé
            colis_prive_auth_url: env::var("COLIS_PRIVE_AUTH_URL")
                .expect("COLIS_PRIVE_AUTH_URL must be set"),
            colis_prive_tournee_url: env::var("COLIS_PRIVE_TOURNEE_URL")
                .expect("COLIS_PRIVE_TOURNEE_URL must be set"),
            colis_prive_detail_url: env::var("COLIS_PRIVE_DETAIL_URL")
                .expect("COLIS_PRIVE_DETAIL_URL must be set"),
            colis_prive_gestion_url: env::var("COLIS_PRIVE_GESTION_URL")
                .expect("COLIS_PRIVE_GESTION_URL must be set"),
            colis_prive_referentiel_url: env::var("COLIS_PRIVE_REFERENTIEL_URL")
                .expect("COLIS_PRIVE_REFERENTIEL_URL must be set"),
        }
    }
}

impl EnvironmentConfig {
    /// Verificar si estamos en modo desarrollo
    pub fn is_development(&self) -> bool {
        self.environment == "development"
    }

    /// Verificar si estamos en modo producción
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }

    /// Obtener la URL del servidor
    pub fn server_url(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

// Las credenciales de Colis Privé ahora se reciben dinámicamente via HTTP requests
// No hay credenciales hardcodeadas en el código

