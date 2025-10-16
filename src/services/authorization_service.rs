use crate::models::auth::{UserRole, UserInfo};
use crate::services::auth_service::AuthService;

/// Servicio de autorización para verificar permisos y roles
pub struct AuthorizationService<'a> {
    auth_service: &'a AuthService,
}

impl<'a> AuthorizationService<'a> {
    pub fn new(auth_service: &'a AuthService) -> Self {
        Self { auth_service }
    }

    /// Verifica si un usuario tiene un rol específico
    pub fn has_role(&self, user_info: &UserInfo, required_role: UserRole) -> bool {
        user_info.role == required_role
    }

    /// Verifica si un usuario tiene al menos uno de los roles requeridos
    pub fn has_any_role(&self, user_info: &UserInfo, required_roles: &[UserRole]) -> bool {
        required_roles.contains(&user_info.role)
    }

    /// Verifica si un usuario tiene todos los roles requeridos
    pub fn has_all_roles(&self, user_info: &UserInfo, required_roles: &[UserRole]) -> bool {
        required_roles.iter().all(|role| user_info.role == *role)
    }

    /// Verifica si un usuario tiene un permiso específico
    pub fn has_permission(&self, user_info: &UserInfo, permission: &str) -> bool {
        user_info.permissions.contains(&permission.to_string())
    }

    /// Verifica si un usuario tiene al menos uno de los permisos requeridos
    pub fn has_any_permission(&self, user_info: &UserInfo, permissions: &[String]) -> bool {
        permissions.iter().any(|perm| user_info.permissions.contains(perm))
    }

    /// Verifica si un usuario tiene todos los permisos requeridos
    pub fn has_all_permissions(&self, user_info: &UserInfo, permissions: &[String]) -> bool {
        permissions.iter().all(|perm| user_info.permissions.contains(perm))
    }

    /// Verifica si un usuario puede acceder a una empresa específica
    pub fn can_access_company(&self, user_info: &UserInfo, company_id: &str) -> bool {
        match user_info.role {
            UserRole::SuperAdmin => true, // Super admin puede acceder a todo
            UserRole::Admin | UserRole::Livreur => {
                user_info.company_id.as_ref().map_or(false, |id| id == company_id)
            }
        }
    }

    /// Verifica si un usuario puede acceder a una tournée específica
    pub fn can_access_tournee(&self, user_info: &UserInfo, tournee_id: &str) -> bool {
        match user_info.role {
            UserRole::SuperAdmin => true, // Super admin puede acceder a todo
            UserRole::Admin => true, // Admin puede acceder a tournées de su empresa
            UserRole::Livreur => {
                user_info.tournee_id.as_ref().map_or(false, |id| id == tournee_id)
            }
        }
    }

    /// Verifica si un usuario puede modificar una tournée específica
    pub fn can_modify_tournee(&self, user_info: &UserInfo, tournee_id: &str) -> bool {
        match user_info.role {
            UserRole::SuperAdmin => true, // Super admin puede modificar todo
            UserRole::Admin => true, // Admin puede modificar tournées de su empresa
            UserRole::Livreur => {
                // Livreur solo puede modificar su propia tournée
                user_info.tournee_id.as_ref().map_or(false, |id| id == tournee_id)
            }
        }
    }

    /// Verifica si un usuario puede ver estadísticas de una empresa
    pub fn can_view_company_stats(&self, user_info: &UserInfo, company_id: &str) -> bool {
        match user_info.role {
            UserRole::SuperAdmin => true, // Super admin puede ver todo
            UserRole::Admin => {
                // Admin solo puede ver estadísticas de su empresa
                user_info.company_id.as_ref().map_or(false, |id| id == company_id)
            }
            UserRole::Livreur => false, // Livreur no puede ver estadísticas
        }
    }

    /// Verifica si un usuario puede gestionar otros usuarios
    pub fn can_manage_users(&self, user_info: &UserInfo) -> bool {
        matches!(user_info.role, UserRole::SuperAdmin | UserRole::Admin)
    }

    /// Verifica si un usuario puede gestionar empresas
    pub fn can_manage_companies(&self, user_info: &UserInfo) -> bool {
        matches!(user_info.role, UserRole::SuperAdmin)
    }

    /// Verifica si un usuario puede ver logs del sistema
    pub fn can_view_system_logs(&self, user_info: &UserInfo) -> bool {
        matches!(user_info.role, UserRole::SuperAdmin)
    }

    /// Verifica si un usuario puede optimizar rutas
    pub fn can_optimize_routes(&self, user_info: &UserInfo) -> bool {
        match user_info.role {
            UserRole::SuperAdmin | UserRole::Admin => true,
            UserRole::Livreur => {
                self.has_permission(user_info, "optimize_route")
            }
        }
    }

    /// Verifica si un usuario puede editar paquetes
    pub fn can_edit_packages(&self, user_info: &UserInfo) -> bool {
        match user_info.role {
            UserRole::SuperAdmin | UserRole::Admin => true,
            UserRole::Livreur => {
                self.has_permission(user_info, "edit_packages")
            }
        }
    }

    /// Verifica si un usuario puede ver paquetes
    pub fn can_view_packages(&self, user_info: &UserInfo) -> bool {
        match user_info.role {
            UserRole::SuperAdmin | UserRole::Admin => true,
            UserRole::Livreur => {
                self.has_permission(user_info, "view_packages")
            }
        }
    }

    /// Obtiene los permisos disponibles para un rol
    pub fn get_permissions_for_role(role: &UserRole) -> Vec<String> {
        match role {
            UserRole::SuperAdmin => vec![
                "view_all_tournees".to_string(),
                "monitor_drivers".to_string(),
                "view_analytics".to_string(),
                "manage_companies".to_string(),
                "manage_users".to_string(),
                "view_system_logs".to_string(),
                "optimize_route".to_string(),
                "edit_packages".to_string(),
                "view_packages".to_string(),
            ],
            UserRole::Admin => vec![
                "view_all_tournees".to_string(),
                "monitor_drivers".to_string(),
                "view_analytics".to_string(),
                "optimize_route".to_string(),
                "edit_packages".to_string(),
                "view_packages".to_string(),
            ],
            UserRole::Livreur => vec![
                "view_packages".to_string(),
                "edit_packages".to_string(),
                "optimize_route".to_string(),
            ],
        }
    }

    /// Verifica si un usuario puede realizar una acción específica
    pub fn can_perform_action(&self, user_info: &UserInfo, action: &str) -> bool {
        match action {
            "view_packages" => self.can_view_packages(user_info),
            "edit_packages" => self.can_edit_packages(user_info),
            "optimize_route" => self.can_optimize_routes(user_info),
            "view_analytics" => matches!(user_info.role, UserRole::SuperAdmin | UserRole::Admin),
            "manage_users" => self.can_manage_users(user_info),
            "manage_companies" => self.can_manage_companies(user_info),
            "view_system_logs" => self.can_view_system_logs(user_info),
            _ => false,
        }
    }

    /// Obtiene el nivel de acceso de un usuario
    pub fn get_access_level(&self, user_info: &UserInfo) -> AccessLevel {
        match user_info.role {
            UserRole::SuperAdmin => AccessLevel::Full,
            UserRole::Admin => AccessLevel::Company,
            UserRole::Livreur => AccessLevel::Tournee,
        }
    }
}

/// Niveles de acceso del sistema
#[derive(Debug, Clone, PartialEq)]
pub enum AccessLevel {
    /// Acceso completo al sistema
    Full,
    /// Acceso a nivel de empresa
    Company,
    /// Acceso a nivel de tournée
    Tournee,
}

impl AccessLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccessLevel::Full => "full",
            AccessLevel::Company => "company",
            AccessLevel::Tournee => "tournee",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_permissions() {
        let auth_service = AuthService::new().await.unwrap();
        let authz_service = AuthorizationService::new(&auth_service);

        let livreur = UserInfo {
            id: "livreur_123".to_string(),
            username: "A187518".to_string(),
            role: UserRole::Livreur,
            company_id: Some("INTI".to_string()),
            tournee_id: Some("A187518".to_string()),
            permissions: vec!["view_packages".to_string(), "edit_packages".to_string()],
        };

        let admin = UserInfo {
            id: "admin_123".to_string(),
            username: "admin_inti".to_string(),
            role: UserRole::Admin,
            company_id: Some("INTI".to_string()),
            tournee_id: None,
            permissions: vec!["view_all_tournees".to_string()],
        };

        // Test role checks
        assert!(authz_service.has_role(&livreur, UserRole::Livreur));
        assert!(!authz_service.has_role(&livreur, UserRole::Admin));

        // Test permission checks
        assert!(authz_service.has_permission(&livreur, "view_packages"));
        assert!(!authz_service.has_permission(&livreur, "manage_users"));

        // Test company access
        assert!(authz_service.can_access_company(&livreur, "INTI"));
        assert!(!authz_service.can_access_company(&livreur, "OTHER"));

        // Test tournee access
        assert!(authz_service.can_access_tournee(&livreur, "A187518"));
        assert!(!authz_service.can_access_tournee(&livreur, "OTHER"));

        // Test admin capabilities
        assert!(authz_service.can_manage_users(&admin));
        assert!(!authz_service.can_manage_companies(&admin));
    }
}
