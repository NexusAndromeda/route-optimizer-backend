use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::Json as ResponseJson,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::models::auth::UserRole;
use crate::services::auth_service::AuthService;
use crate::services::authorization_service::AuthorizationService;

/// Estado compartido de la aplicación
pub type AppState = Arc<Mutex<AuthService>>;

/// Obtiene los permisos de un usuario
pub async fn get_user_permissions(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let auth_service = state.lock().await;
    
    // En una implementación real, buscaríamos el usuario por ID
    // Por simplicidad, retornamos un error
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Obtiene los roles disponibles en el sistema
pub async fn get_available_roles() -> Result<ResponseJson<Value>, StatusCode> {
    let roles = vec![
        json!({
            "role": "super_admin",
            "name": "Super Administrador",
            "description": "Acceso completo al sistema",
            "permissions": AuthorizationService::get_permissions_for_role(&UserRole::SuperAdmin)
        }),
        json!({
            "role": "admin",
            "name": "Administrador",
            "description": "Acceso a nivel de empresa",
            "permissions": AuthorizationService::get_permissions_for_role(&UserRole::Admin)
        }),
        json!({
            "role": "livreur",
            "name": "Livreur",
            "description": "Acceso a nivel de tournée",
            "permissions": AuthorizationService::get_permissions_for_role(&UserRole::Livreur)
        }),
    ];

    Ok(ResponseJson(json!({
        "success": true,
        "data": {
            "roles": roles
        },
        "message": "Roles disponibles"
    })))
}

/// Verifica si un usuario puede realizar una acción específica
pub async fn check_permission(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let token = payload.get("token")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let action = payload.get("action")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let auth_service = state.lock().await;
    
        match auth_service.validate_token(token) {
            Ok(user_info) => {
                let authz_service = AuthorizationService::new(&auth_service);
                let can_perform = authz_service.can_perform_action(&user_info, action);
            
            Ok(ResponseJson(json!({
                "success": true,
                "data": {
                    "can_perform": can_perform,
                    "user_role": user_info.role.as_str(),
                    "action": action
                },
                "message": if can_perform { "Acceso permitido" } else { "Acceso denegado" }
            })))
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Obtiene el nivel de acceso de un usuario
pub async fn get_access_level(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let token = payload.get("token")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let auth_service = state.lock().await;
    
        match auth_service.validate_token(token) {
            Ok(user_info) => {
                let authz_service = AuthorizationService::new(&auth_service);
                let access_level = authz_service.get_access_level(&user_info);
            
            Ok(ResponseJson(json!({
                "success": true,
                "data": {
                    "access_level": access_level.as_str(),
                    "user_role": user_info.role.as_str(),
                    "company_id": user_info.company_id,
                    "tournee_id": user_info.tournee_id
                },
                "message": "Nivel de acceso obtenido"
            })))
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Verifica si un usuario puede acceder a una empresa específica
pub async fn check_company_access(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let token = payload.get("token")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let company_id = payload.get("company_id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let auth_service = state.lock().await;
    
        match auth_service.validate_token(token) {
            Ok(user_info) => {
                let authz_service = AuthorizationService::new(&auth_service);
                let can_access = authz_service.can_access_company(&user_info, company_id);
            
            Ok(ResponseJson(json!({
                "success": true,
                "data": {
                    "can_access": can_access,
                    "company_id": company_id,
                    "user_company_id": user_info.company_id
                },
                "message": if can_access { "Acceso permitido" } else { "Acceso denegado" }
            })))
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Verifica si un usuario puede acceder a una tournée específica
pub async fn check_tournee_access(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let token = payload.get("token")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let tournee_id = payload.get("tournee_id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let auth_service = state.lock().await;
    
        match auth_service.validate_token(token) {
            Ok(user_info) => {
                let authz_service = AuthorizationService::new(&auth_service);
                let can_access = authz_service.can_access_tournee(&user_info, tournee_id);
            
            Ok(ResponseJson(json!({
                "success": true,
                "data": {
                    "can_access": can_access,
                    "tournee_id": tournee_id,
                    "user_tournee_id": user_info.tournee_id
                },
                "message": if can_access { "Acceso permitido" } else { "Acceso denegado" }
            })))
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Obtiene las estadísticas de permisos del sistema
pub async fn get_permission_stats() -> Result<ResponseJson<Value>, StatusCode> {
    let stats = json!({
        "total_roles": 3,
        "total_permissions": 9,
        "role_distribution": {
            "super_admin": {
                "permissions": 9,
                "access_level": "full"
            },
            "admin": {
                "permissions": 6,
                "access_level": "company"
            },
            "livreur": {
                "permissions": 3,
                "access_level": "tournee"
            }
        },
        "permission_categories": {
            "system_management": ["manage_companies", "manage_users", "view_system_logs"],
            "data_access": ["view_all_tournees", "view_analytics", "view_packages"],
            "data_modification": ["edit_packages", "optimize_route"],
            "monitoring": ["monitor_drivers"]
        }
    });

    Ok(ResponseJson(json!({
        "success": true,
        "data": stats,
        "message": "Estadísticas de permisos obtenidas"
    })))
}
