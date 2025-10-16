use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::Json as ResponseJson,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::models::auth::{LoginRequest, RefreshTokenRequest};
use crate::services::auth_service::AuthService;

/// Estado compartido de la aplicación
pub type AppState = Arc<Mutex<AuthService>>;

/// Endpoint de login
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let mut auth_service = state.lock().await;
    
    match auth_service.authenticate(&payload).await {
        Ok(response) => {
            if response.success {
                Ok(ResponseJson(json!({
                    "success": true,
                    "data": {
                        "token": response.token,
                        "user_info": response.user_info,
                        "expires_at": response.expires_at
                    },
                    "message": response.message
                })))
            } else {
                Ok(ResponseJson(json!({
                    "success": false,
                    "message": response.message
                })))
            }
        }
        Err(e) => {
            log::error!("Login error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Endpoint de refresh token
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let auth_service = state.lock().await;
    
    match auth_service.refresh_token(&payload) {
        Ok(response) => {
            if response.success {
                Ok(ResponseJson(json!({
                    "success": true,
                    "data": {
                        "token": response.token,
                        "expires_at": response.expires_at
                    },
                    "message": response.message
                })))
            } else {
                Ok(ResponseJson(json!({
                    "success": false,
                    "message": response.message
                })))
            }
        }
        Err(e) => {
            log::error!("Token refresh error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Endpoint de logout
pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let mut auth_service = state.lock().await;
    
    let user_id = payload.get("user_id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let success = auth_service.logout(user_id);
    
    Ok(ResponseJson(json!({
        "success": success,
        "message": if success { "Logout successful" } else { "User not found" }
    })))
}

/// Endpoint de validación de token
pub async fn validate_token(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let auth_service = state.lock().await;
    
    let token = payload.get("token")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    match auth_service.validate_token(token) {
        Ok(user_info) => {
            Ok(ResponseJson(json!({
                "success": true,
                "data": {
                    "user_info": user_info
                },
                "message": "Token valid"
            })))
        }
        Err(e) => {
            Ok(ResponseJson(json!({
                "success": false,
                "message": format!("Invalid token: {}", e)
            })))
        }
    }
}

/// Endpoint para obtener sesiones activas (solo para admins)
pub async fn get_active_sessions(
    State(state): State<AppState>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let auth_service = state.lock().await;
    let sessions = auth_service.get_active_sessions();
    
    Ok(ResponseJson(json!({
        "success": true,
        "data": {
            "sessions": sessions
        },
        "message": "Active sessions retrieved"
    })))
}

/// Middleware para validar JWT en headers
pub async fn validate_auth_header(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<ResponseJson<Value>, StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let token = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    
    let auth_service = state.lock().await;
    
    match auth_service.validate_token(token) {
        Ok(user_info) => {
            // Actualizar última actividad
            drop(auth_service);
            let mut auth_service = state.lock().await;
            auth_service.update_last_activity(&user_info.id);
            
            Ok(ResponseJson(json!({
                "success": true,
                "data": {
                    "user_info": user_info
                },
                "message": "Authentication successful"
            })))
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
