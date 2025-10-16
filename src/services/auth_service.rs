use crate::models::auth::{
    LoginRequest, LoginResponse, UserInfo, UserRole, UserType, 
    RefreshTokenRequest, RefreshTokenResponse, SessionInfo
};
use crate::services::jwt_service::JwtService;
use crate::services::colis_prive_service::ColisPriveService;
use crate::services::ssshops_cache_service::SsshopsCacheService;
use crate::config::environment::EnvironmentConfig;
use crate::cache::redis_client::RedisClient;
use reqwest::Client;
use std::collections::HashMap;
use chrono::Utc;
use bcrypt::{hash, verify, DEFAULT_COST};
use anyhow::Result;

/// Servicio de autenticación
pub struct AuthService {
    jwt_service: JwtService,
    colis_prive_service: ColisPriveService,
    ssshops_cache: SsshopsCacheService,
    // Cache de sesiones activas
    active_sessions: HashMap<String, SessionInfo>,
    // Base de datos de admins (en producción sería una BD real)
    admin_users: HashMap<String, AdminUser>,
}

#[derive(Debug, Clone)]
struct AdminUser {
    id: String,
    username: String,
    password_hash: String,
    company_id: String,
    role: UserRole,
    permissions: Vec<String>,
}

impl AuthService {
    pub async fn new() -> Result<Self> {
        let client = Client::new();
        let config = EnvironmentConfig::default();
        
        // Crear cliente Redis (en producción usaría configuración real)
        let redis_config = crate::cache::cache_config::CacheConfig::default();
        let redis = RedisClient::new(redis_config).await?;
        let ssshops_cache = SsshopsCacheService::new(redis);
        
        let mut service = Self {
            jwt_service: JwtService::new(),
            colis_prive_service: ColisPriveService::new(client, config),
            ssshops_cache,
            active_sessions: HashMap::new(),
            admin_users: HashMap::new(),
        };
        
        // Inicializar usuarios admin de ejemplo
        service.initialize_admin_users();
        Ok(service)
    }

    /// Inicializa usuarios admin de ejemplo
    fn initialize_admin_users(&mut self) {
        // Admin de INTI
        let inti_admin = AdminUser {
            id: "admin_inti_001".to_string(),
            username: "admin_inti".to_string(),
            password_hash: hash("admin123", DEFAULT_COST).unwrap(),
            company_id: "INTI".to_string(),
            role: UserRole::Admin,
            permissions: vec![
                "view_all_tournees".to_string(),
                "monitor_drivers".to_string(),
                "view_analytics".to_string(),
            ],
        };

        // Super admin
        let super_admin = AdminUser {
            id: "super_admin_001".to_string(),
            username: "super_admin".to_string(),
            password_hash: hash("super123", DEFAULT_COST).unwrap(),
            company_id: "ALL".to_string(),
            role: UserRole::SuperAdmin,
            permissions: vec![
                "view_all_tournees".to_string(),
                "monitor_drivers".to_string(),
                "view_analytics".to_string(),
                "manage_companies".to_string(),
                "manage_users".to_string(),
            ],
        };

        self.admin_users.insert("admin_inti".to_string(), inti_admin);
        self.admin_users.insert("super_admin".to_string(), super_admin);
    }

    /// Autentica un usuario (livreur o admin)
    pub async fn authenticate(&mut self, request: &LoginRequest) -> Result<LoginResponse> {
        match request.user_type {
            UserType::Livreur => self.authenticate_livreur(request).await,
            UserType::Admin => self.authenticate_admin(request).await,
        }
    }

    /// Autentica un livreur usando Colis Privé
    async fn authenticate_livreur(&mut self, request: &LoginRequest) -> Result<LoginResponse> {
        let company_id = "INTI"; // Por defecto
        
        // Primero verificar si ya tenemos un token válido en caché
        if let Ok(Some(cached_token)) = self.ssshops_cache.get_ssshops_token(company_id, &request.username).await {
            log::info!("✅ Usando token Ssshops en caché para {}:{}", company_id, request.username);
            return self.create_livreur_response(request, company_id, &cached_token.token).await;
        }
        
        // Si no hay token en caché, autenticar con Colis Privé
        match self.colis_prive_service.authenticate(&request.username, &request.password, company_id).await {
            Ok(auth_response) => {
                // Si tenemos token SSO, la autenticación fue exitosa
                if !auth_response.sso_token.is_empty() {
                    // Cachear el token para futuras consultas
                    if let Err(e) = self.ssshops_cache.cache_ssshops_token(
                        company_id,
                        &request.username,
                        &auth_response.sso_token,
                        24, // 24 horas
                    ).await {
                        log::warn!("⚠️ No se pudo cachear token Ssshops: {}", e);
                    }
                    
                    return self.create_livreur_response(request, company_id, &auth_response.sso_token).await;
                } else {
                    Ok(LoginResponse {
                        success: false,
                        token: None,
                        user_info: None,
                        message: Some("Invalid credentials".to_string()),
                        expires_at: None,
                    })
                }
            }
            Err(e) => {
                Ok(LoginResponse {
                    success: false,
                    token: None,
                    user_info: None,
                    message: Some(format!("Authentication error: {}", e)),
                    expires_at: None,
                })
            }
        }
    }

    /// Autentica un admin usando base de datos local
    async fn authenticate_admin(&mut self, request: &LoginRequest) -> Result<LoginResponse> {
        if let Some(admin_user) = self.admin_users.get(&request.username) {
            // Verificar contraseña
            if verify(&request.password, &admin_user.password_hash)? {
                let user_info = UserInfo {
                    id: admin_user.id.clone(),
                    username: admin_user.username.clone(),
                    role: admin_user.role.clone(),
                    company_id: Some(admin_user.company_id.clone()),
                    tournee_id: None, // Los admins no tienen tournee_id
                    permissions: admin_user.permissions.clone(),
                };

                // Generar JWT
                let token = self.jwt_service.generate_access_token(&user_info)
                    .map_err(|e| anyhow::anyhow!("JWT generation failed: {}", e))?;
                let expires_at = Utc::now() + chrono::Duration::hours(24);

                // Guardar sesión
                let session = SessionInfo {
                    user_id: user_info.id.clone(),
                    username: user_info.username.clone(),
                    role: user_info.role.clone(),
                    company_id: user_info.company_id.clone(),
                    tournee_id: user_info.tournee_id.clone(),
                    created_at: Utc::now(),
                    last_activity: Utc::now(),
                    is_active: true,
                };
                self.active_sessions.insert(user_info.id.clone(), session);

                Ok(LoginResponse {
                    success: true,
                    token: Some(token),
                    user_info: Some(user_info),
                    message: Some("Admin login successful".to_string()),
                    expires_at: Some(expires_at),
                })
            } else {
                Ok(LoginResponse {
                    success: false,
                    token: None,
                    user_info: None,
                    message: Some("Invalid admin credentials".to_string()),
                    expires_at: None,
                })
            }
        } else {
            Ok(LoginResponse {
                success: false,
                token: None,
                user_info: None,
                message: Some("Admin user not found".to_string()),
                expires_at: None,
            })
        }
    }

    /// Refresca un token
    pub fn refresh_token(&self, request: &RefreshTokenRequest) -> Result<RefreshTokenResponse> {
        match self.jwt_service.refresh_access_token(&request.token) {
            Ok(new_token) => {
                let expires_at = Utc::now() + chrono::Duration::hours(24);
                Ok(RefreshTokenResponse {
                    success: true,
                    token: Some(new_token),
                    expires_at: Some(expires_at),
                    message: Some("Token refreshed successfully".to_string()),
                })
            }
            Err(e) => {
                Ok(RefreshTokenResponse {
                    success: false,
                    token: None,
                    expires_at: None,
                    message: Some(format!("Token refresh failed: {}", e)),
                })
            }
        }
    }

    /// Valida un token y devuelve la información del usuario
    pub fn validate_token(&self, token: &str) -> Result<UserInfo> {
        self.jwt_service.get_user_info(token)
            .map_err(|e| anyhow::anyhow!("Token validation failed: {}", e))
    }

    /// Cierra una sesión
    pub fn logout(&mut self, user_id: &str) -> bool {
        self.active_sessions.remove(user_id).is_some()
    }

    /// Obtiene información de una sesión activa
    pub fn get_session(&self, user_id: &str) -> Option<&SessionInfo> {
        self.active_sessions.get(user_id)
    }

    /// Actualiza la última actividad de una sesión
    pub fn update_last_activity(&mut self, user_id: &str) {
        if let Some(session) = self.active_sessions.get_mut(user_id) {
            session.last_activity = Utc::now();
        }
    }

    /// Obtiene todas las sesiones activas (para monitoreo)
    pub fn get_active_sessions(&self) -> Vec<&SessionInfo> {
        self.active_sessions.values().collect()
    }

    /// Verifica si un usuario tiene un permiso específico
    pub fn has_permission(&self, token: &str, permission: &str) -> Result<bool> {
        self.jwt_service.has_permission(token, permission)
            .map_err(|e| anyhow::anyhow!("Permission check failed: {}", e))
    }

    /// Crea la respuesta de login para un livreur
    async fn create_livreur_response(
        &mut self,
        request: &LoginRequest,
        company_id: &str,
        _ssshops_token: &str,
    ) -> Result<LoginResponse> {
        let user_info = UserInfo {
            id: format!("livreur_{}", request.username),
            username: request.username.clone(),
            role: UserRole::Livreur,
            company_id: Some(company_id.to_string()),
            tournee_id: Some(request.username.clone()), // tournee_id = username
            permissions: vec![
                "view_packages".to_string(),
                "edit_packages".to_string(),
                "optimize_route".to_string(),
            ],
        };

        // Generar JWT
        let token = self.jwt_service.generate_access_token(&user_info)
            .map_err(|e| anyhow::anyhow!("JWT generation failed: {}", e))?;
        let expires_at = Utc::now() + chrono::Duration::hours(24);

        // Guardar sesión
        let session = SessionInfo {
            user_id: user_info.id.clone(),
            username: user_info.username.clone(),
            role: user_info.role.clone(),
            company_id: user_info.company_id.clone(),
            tournee_id: user_info.tournee_id.clone(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
            is_active: true,
        };
        self.active_sessions.insert(user_info.id.clone(), session);

        Ok(LoginResponse {
            success: true,
            token: Some(token),
            user_info: Some(user_info),
            message: Some("Login successful".to_string()),
            expires_at: Some(expires_at),
        })
    }
}

// No podemos implementar Default porque new() es async
// En su lugar, usaremos AuthService::new() directamente donde sea necesario

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_admin_authentication() {
        let mut auth_service = AuthService::new();
        
        let request = LoginRequest {
            username: "admin_inti".to_string(),
            password: "admin123".to_string(),
            user_type: UserType::Admin,
        };

        let response = auth_service.authenticate(&request).await.unwrap();
        assert!(response.success);
        assert!(response.token.is_some());
        assert!(response.user_info.is_some());
        
        let user_info = response.user_info.unwrap();
        assert_eq!(user_info.username, "admin_inti");
        assert_eq!(user_info.role, UserRole::Admin);
        assert_eq!(user_info.company_id, Some("INTI".to_string()));
    }

    #[tokio::test]
    async fn test_invalid_admin_authentication() {
        let mut auth_service = AuthService::new();
        
        let request = LoginRequest {
            username: "admin_inti".to_string(),
            password: "wrong_password".to_string(),
            user_type: UserType::Admin,
        };

        let response = auth_service.authenticate(&request).await.unwrap();
        assert!(!response.success);
        assert!(response.token.is_none());
    }
}
