//! Utilidades JWT
//! 
//! Este módulo contiene funciones helper para manejo de JWT tokens
//! y verificación de autenticación.

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    config::EnvironmentConfig,
    models::user::UserType,
    utils::errors::AppError,
};

/// Claims del JWT token
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,        // user_id
    pub company_id: String, // company_id
    pub user_type: String,  // user_type
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
    user_type: UserType,
    config: &JwtConfig,
) -> Result<String, AppError> {
    let now = chrono::Utc::now();
    let expires_at = now + chrono::Duration::seconds(config.expiration as i64);

    let claims = JwtClaims {
        sub: user_id.to_string(),
        company_id: company_id.to_string(),
        user_type: format!("{:?}", user_type).to_lowercase(),
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

/// Generar token de refresh (opcional)
pub fn generate_refresh_token(
    user_id: Uuid,
    company_id: Uuid,
    config: &JwtConfig,
) -> Result<String, AppError> {
    let now = chrono::Utc::now();
    // Refresh token expira en 30 días
    let expires_at = now + chrono::Duration::days(30);

    let claims = JwtClaims {
        sub: user_id.to_string(),
        company_id: company_id.to_string(),
        user_type: "refresh".to_string(),
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let encoding_key = EncodingKey::from_secret(config.secret.as_ref());
    
    encode(&Header::default(), &claims, &encoding_key)
        .map_err(|e| AppError::Jwt(format!("Error generando refresh token: {}", e)))
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

/// Crear respuesta de autenticación exitosa
pub fn create_auth_response(
    access_token: String,
    refresh_token: Option<String>,
    user_id: Uuid,
    company_id: Uuid,
    user_type: UserType,
    config: &JwtConfig,
) -> serde_json::Value {
    let mut response = json!({
        "access_token": access_token,
        "token_type": "Bearer",
        "expires_in": config.expiration,
        "user_id": user_id.to_string(),
        "company_id": company_id.to_string(),
        "user_type": format!("{:?}", user_type).to_lowercase(),
    });

    if let Some(refresh_token) = refresh_token {
        response["refresh_token"] = serde_json::Value::String(refresh_token);
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EnvironmentConfig;

    fn create_test_config() -> JwtConfig {
        JwtConfig {
            secret: "test-secret-key".to_string(),
            expiration: 3600, // 1 hora
            issuer: None,
            audience: None,
        }
    }

    #[test]
    fn test_generate_and_verify_token() {
        let config = create_test_config();
        let user_id = Uuid::new_v4();
        let company_id = Uuid::new_v4();
        let user_type = UserType::Admin;

        let token = generate_token(user_id, company_id, user_type, &config).unwrap();
        let claims = verify_token(&token, &config).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.company_id, company_id.to_string());
        assert_eq!(claims.user_type, "admin");
    }

    #[test]
    fn test_token_expiration() {
        let config = JwtConfig {
            secret: "test-secret".to_string(),
            expiration: 1, // 1 segundo
            issuer: None,
            audience: None,
        };

        let user_id = Uuid::new_v4();
        let company_id = Uuid::new_v4();
        let user_type = UserType::Driver;

        let token = generate_token(user_id, company_id, user_type, &config).unwrap();
        
        // Esperar a que expire
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        let result = verify_token(&token, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_token_format() {
        assert!(validate_token_format("header.payload.signature").is_ok());
        assert!(validate_token_format("invalid-token").is_err());
        assert!(validate_token_format("").is_err());
    }

    #[test]
    fn test_extract_token_from_header() {
        assert_eq!(
            extract_token_from_header("Bearer valid-token").unwrap(),
            "valid-token"
        );
        assert!(extract_token_from_header("Invalid header").is_err());
        assert!(extract_token_from_header("Bearer ").is_err());
    }
}
