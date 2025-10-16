use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use crate::services::auth_service::AuthService;
use crate::models::auth::UserRole;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Estado compartido de la aplicación
pub type AppState = Arc<Mutex<AuthService>>;

/// Middleware de autenticación
pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extraer token del header Authorization
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    // Validar token
    let auth_service = state.lock().await;
    match auth_service.validate_token(token) {
        Ok(user_info) => {
            // Agregar información del usuario al request
            // En una implementación real, usaríamos extensiones de Axum
            drop(auth_service);
            Ok(next.run(request).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Middleware de autorización por roles
pub async fn role_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
    required_role: UserRole,
) -> Result<Response, StatusCode> {
    // Extraer token del header Authorization
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    // Validar token y verificar rol
    let auth_service = state.lock().await;
    match auth_service.validate_token(token) {
        Ok(user_info) => {
            if user_info.role == required_role {
                drop(auth_service);
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Middleware de autorización por permisos
pub async fn permission_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
    required_permission: &str,
) -> Result<Response, StatusCode> {
    // Extraer token del header Authorization
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    // Validar token y verificar permiso
    let auth_service = state.lock().await;
    match auth_service.validate_token(token) {
        Ok(user_info) => {
            if user_info.permissions.contains(&required_permission.to_string()) {
                drop(auth_service);
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Middleware para verificar si el usuario es admin
pub async fn admin_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    role_middleware(State(state), headers, request, next, UserRole::Admin).await
}

/// Middleware para verificar si el usuario es livreur
pub async fn livreur_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    role_middleware(State(state), headers, request, next, UserRole::Livreur).await
}

/// Middleware para verificar si el usuario es super admin
pub async fn super_admin_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    role_middleware(State(state), headers, request, next, UserRole::SuperAdmin).await
}

/// Helper para extraer información del usuario del token
pub fn extract_user_info_from_token(
    auth_service: &AuthService,
    token: &str,
) -> Result<crate::models::auth::UserInfo, StatusCode> {
    auth_service.validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)
}

/// Helper para verificar si un usuario tiene un permiso específico
pub fn check_permission(
    auth_service: &AuthService,
    token: &str,
    permission: &str,
) -> Result<bool, StatusCode> {
    auth_service.has_permission(token, permission)
        .map_err(|_| StatusCode::UNAUTHORIZED)
}

/// Helper para verificar si un usuario tiene un rol específico
pub fn check_role(
    auth_service: &AuthService,
    token: &str,
    required_role: UserRole,
) -> Result<bool, StatusCode> {
    match auth_service.validate_token(token) {
        Ok(user_info) => Ok(user_info.role == required_role),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Helper para verificar si un usuario puede acceder a una empresa específica
pub fn check_company_access(
    auth_service: &AuthService,
    token: &str,
    company_id: &str,
) -> Result<bool, StatusCode> {
    match auth_service.validate_token(token) {
        Ok(user_info) => {
            match user_info.role {
                UserRole::SuperAdmin => Ok(true), // Super admin puede acceder a todo
                UserRole::Admin => {
                    // Admin solo puede acceder a su empresa
                    Ok(user_info.company_id.as_ref().map_or(false, |id| id == company_id))
                }
                UserRole::Livreur => {
                    // Livreur solo puede acceder a su empresa
                    Ok(user_info.company_id.as_ref().map_or(false, |id| id == company_id))
                }
            }
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Helper para verificar si un usuario puede acceder a una tournée específica
pub fn check_tournee_access(
    auth_service: &AuthService,
    token: &str,
    tournee_id: &str,
) -> Result<bool, StatusCode> {
    match auth_service.validate_token(token) {
        Ok(user_info) => {
            match user_info.role {
                UserRole::SuperAdmin => Ok(true), // Super admin puede acceder a todo
                UserRole::Admin => {
                    // Admin puede acceder a tournées de su empresa
                    Ok(true) // En una implementación real, verificaríamos la empresa de la tournée
                }
                UserRole::Livreur => {
                    // Livreur solo puede acceder a su propia tournée
                    Ok(user_info.tournee_id.as_ref().map_or(false, |id| id == tournee_id))
                }
            }
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
