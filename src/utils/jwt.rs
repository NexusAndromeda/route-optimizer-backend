//! Utilidades JWT Simplificadas
//! 
//! Este módulo contiene funciones helper para manejo de JWT tokens simplificados.

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::environment::EnvironmentConfig,
    utils::errors::AppError,
};

/// Claims del JWT token simplificado
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,        // user_id
    pub company_id: String, // company_id
    pub exp: usize,         // expiration timestamp
    pub iat: usize,         // issued at timestamp
}

/// Configuración de JWT
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration: u64,
    pub issuer: Option<String>,
    pub audience: Option<String>,
}

impl From<&EnvironmentConfig> for JwtConfig {
    fn from(config: &EnvironmentConfig) -> Self {
        Self {
            secret: config.jwt_secret.clone(),
            expiration: config.jwt_expiration,
            issuer: None,
            audience: None,
        }
    }
}

/// Generar JWT token para un usuario
pub fn generate_token(
    user_id: Uuid,
    company_id: Uuid,
    config: &JwtConfig,
) -> Result<String, AppError> {
    let now = chrono::Utc::now();
    let expires_at = now + chrono::Duration::seconds(config.expiration as i64);

    let claims = JwtClaims {
        sub: user_id.to_string(),
        company_id: company_id.to_string(),
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let encoding_key = EncodingKey::from_secret(config.secret.as_ref());
    
    encode(&Header::default(), &claims, &encoding_key)
        .map_err(|e| AppError::Jwt(format!("Error generando token: {}", e)))
}

/// Verificar y decodificar JWT token
pub fn verify_token(token: &str, config: &JwtConfig) -> Result<JwtClaims, AppError> {
    let decoding_key = DecodingKey::from_secret(config.secret.as_ref());
    
    let token_data = decode::<JwtClaims>(
        token,
        &decoding_key,
        &Validation::default(),
    )
    .map_err(|e| AppError::Jwt(format!("Token inválido: {}", e)))?;

    Ok(token_data.claims)
}

/// Verificar si un token ha expirado
pub fn is_token_expired(claims: &JwtClaims) -> bool {
    let now = chrono::Utc::now().timestamp() as usize;
    claims.exp < now
}

/// Obtener tiempo restante de un token
pub fn get_token_remaining_time(claims: &JwtClaims) -> i64 {
    let now = chrono::Utc::now().timestamp() as usize;
    if claims.exp > now {
        (claims.exp - now) as i64
    } else {
        0
    }
}

/// Validar formato de token (básico)
pub fn validate_token_format(token: &str) -> Result<(), AppError> {
    if token.is_empty() {
        return Err(AppError::Jwt("Token no puede estar vacío".to_string()));
    }

    if !token.contains('.') {
        return Err(AppError::Jwt("Formato de token inválido".to_string()));
    }

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(AppError::Jwt("Token debe tener 3 partes separadas por puntos".to_string()));
    }

    Ok(())
}

/// Extraer token del header Authorization
pub fn extract_token_from_header(auth_header: &str) -> Result<&str, AppError> {
    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Jwt("Header Authorization debe comenzar con 'Bearer '".to_string()));
    }

    let token = &auth_header[7..]; // Remover "Bearer "
    if token.is_empty() {
        return Err(AppError::Jwt("Token no puede estar vacío".to_string()));
    }

    Ok(token)
}