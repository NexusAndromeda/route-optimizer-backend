use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use std::env;
use crate::models::auth::{JwtClaims, UserInfo, UserRole};

/// Configuración JWT
pub struct JwtConfig {
    pub secret: String,
    pub algorithm: Algorithm,
    pub access_token_duration: Duration,
    pub refresh_token_duration: Duration,
}

impl JwtConfig {
    pub fn new() -> Self {
        let secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string());
        
        Self {
            secret,
            algorithm: Algorithm::HS256,
            access_token_duration: Duration::hours(24), // 24 horas
            refresh_token_duration: Duration::days(7),   // 7 días
        }
    }
}

/// Servicio JWT
pub struct JwtService {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new() -> Self {
        let config = JwtConfig::new();
        let encoding_key = EncodingKey::from_secret(config.secret.as_ref());
        let decoding_key = DecodingKey::from_secret(config.secret.as_ref());
        
        Self {
            config,
            encoding_key,
            decoding_key,
        }
    }

    /// Genera un token de acceso
    pub fn generate_access_token(&self, user_info: &UserInfo) -> Result<String, String> {
        let now = Utc::now();
        let exp = now + self.config.access_token_duration;
        
        let claims = JwtClaims {
            sub: user_info.id.clone(),
            username: user_info.username.clone(),
            role: user_info.role.as_str().to_string(),
            company_id: user_info.company_id.clone(),
            tournee_id: user_info.tournee_id.clone(),
            permissions: user_info.permissions.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(&Header::new(self.config.algorithm), &claims, &self.encoding_key)
            .map_err(|e| format!("Error generating access token: {}", e))
    }

    /// Genera un token de refresh
    pub fn generate_refresh_token(&self, user_info: &UserInfo) -> Result<String, String> {
        let now = Utc::now();
        let exp = now + self.config.refresh_token_duration;
        
        let claims = JwtClaims {
            sub: user_info.id.clone(),
            username: user_info.username.clone(),
            role: user_info.role.as_str().to_string(),
            company_id: user_info.company_id.clone(),
            tournee_id: user_info.tournee_id.clone(),
            permissions: user_info.permissions.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(&Header::new(self.config.algorithm), &claims, &self.encoding_key)
            .map_err(|e| format!("Error generating refresh token: {}", e))
    }

    /// Valida y decodifica un token
    pub fn validate_token(&self, token: &str) -> Result<JwtClaims, String> {
        let validation = Validation::new(self.config.algorithm);
        
        decode::<JwtClaims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| format!("Invalid token: {}", e))
    }

    /// Verifica si un token está expirado
    pub fn is_token_expired(&self, token: &str) -> bool {
        match self.validate_token(token) {
            Ok(claims) => {
                let now = Utc::now().timestamp();
                now >= claims.exp
            }
            Err(_) => true, // Si no se puede decodificar, considerarlo expirado
        }
    }

    /// Refresca un token (genera nuevo access token)
    pub fn refresh_access_token(&self, refresh_token: &str) -> Result<String, String> {
        let claims = self.validate_token(refresh_token)?;
        
        // Verificar que el token no esté expirado
        if self.is_token_expired(refresh_token) {
            return Err("Refresh token expired".to_string());
        }

        // Crear nuevo UserInfo desde los claims
        let user_info = UserInfo {
            id: claims.sub,
            username: claims.username,
            role: UserRole::from_str(&claims.role)
                .ok_or("Invalid role in token")?,
            company_id: claims.company_id,
            tournee_id: claims.tournee_id,
            permissions: claims.permissions,
        };

        // Generar nuevo access token
        self.generate_access_token(&user_info)
    }

    /// Extrae el user_id del token
    pub fn get_user_id(&self, token: &str) -> Result<String, String> {
        let claims = self.validate_token(token)?;
        Ok(claims.sub)
    }

    /// Extrae el role del token
    pub fn get_user_role(&self, token: &str) -> Result<UserRole, String> {
        let claims = self.validate_token(token)?;
        UserRole::from_str(&claims.role)
            .ok_or("Invalid role in token".to_string())
    }

    /// Verifica si el usuario tiene un permiso específico
    pub fn has_permission(&self, token: &str, permission: &str) -> Result<bool, String> {
        let claims = self.validate_token(token)?;
        Ok(claims.permissions.contains(&permission.to_string()))
    }

    /// Obtiene información completa del usuario desde el token
    pub fn get_user_info(&self, token: &str) -> Result<UserInfo, String> {
        let claims = self.validate_token(token)?;
        
        Ok(UserInfo {
            id: claims.sub,
            username: claims.username,
            role: UserRole::from_str(&claims.role)
                .ok_or("Invalid role in token")?,
            company_id: claims.company_id,
            tournee_id: claims.tournee_id,
            permissions: claims.permissions,
        })
    }
}

impl Default for JwtService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_generate_and_validate_token() {
        let jwt_service = JwtService::new();
        
        let user_info = UserInfo {
            id: "test_user_123".to_string(),
            username: "test_user".to_string(),
            role: UserRole::Livreur,
            company_id: Some("INTI".to_string()),
            tournee_id: Some("A187518".to_string()),
            permissions: vec!["view_packages".to_string(), "edit_packages".to_string()],
        };

        // Generar token
        let token = jwt_service.generate_access_token(&user_info).unwrap();
        assert!(!token.is_empty());

        // Validar token
        let claims = jwt_service.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "test_user_123");
        assert_eq!(claims.username, "test_user");
        assert_eq!(claims.role, "livreur");
    }

    #[test]
    fn test_token_expiration() {
        let jwt_service = JwtService::new();
        
        let user_info = UserInfo {
            id: "test_user_123".to_string(),
            username: "test_user".to_string(),
            role: UserRole::Livreur,
            company_id: Some("INTI".to_string()),
            tournee_id: Some("A187518".to_string()),
            permissions: vec![],
        };

        let token = jwt_service.generate_access_token(&user_info).unwrap();
        
        // Token recién creado no debería estar expirado
        assert!(!jwt_service.is_token_expired(&token));
    }
}
