//! Middleware de autenticación JWT Simplificado
//! 
//! Este módulo maneja la autenticación JWT simplificada para el schema simplificado.

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
    Extension,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::environment::EnvironmentConfig,
    utils::errors::AppError,
};

/// Claims del JWT simplificado
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub company_id: String,
    pub exp: usize,
    pub iat: usize,
}

/// Usuario autenticado que se inyecta en las requests
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub company_id: Uuid,
}

/// Middleware de autenticación JWT
pub async fn auth_middleware(
    State(pool): State<PgPool>,
    State(config): State<EnvironmentConfig>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extraer token del header Authorization
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_str| auth_str.to_str().ok())
        .and_then(|auth_str| auth_str.strip_prefix("Bearer "))
        .ok_or_else(|| {
            AppError::Unauthorized("Token de autorización requerido".to_string())
        })?;

    // Decodificar y validar JWT
    let token_data = decode::<Claims>(
        auth_header,
        &DecodingKey::from_secret(config.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Token inválido".to_string()))?;

    let claims = token_data.claims;

    // Verificar que el usuario existe en la base de datos
    let user_row = sqlx::query!(
        r#"
        SELECT 
            id, company_id, password_hash, full_name, email, created_at
        FROM users 
        WHERE id = $1 
        AND company_id = $2
        "#,
        Uuid::parse_str(&claims.sub).map_err(|_| {
            AppError::Unauthorized("ID de usuario inválido".to_string())
        })?,
        Uuid::parse_str(&claims.company_id).map_err(|_| {
            AppError::Unauthorized("ID de empresa inválido".to_string())
        })?
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e))?
    .ok_or_else(|| AppError::Unauthorized("Usuario no encontrado".to_string()))?;

    // Crear usuario autenticado
    let authenticated_user = AuthenticatedUser {
        user_id: user_row.id,
        company_id: user_row.company_id,
    };

    // Inyectar usuario autenticado en las extensions
    request.extensions_mut().insert(authenticated_user);

    Ok(next.run(request).await)
}

/// Middleware opcional de autenticación (para rutas que pueden ser públicas o privadas)
pub async fn optional_auth_middleware(
    State(pool): State<PgPool>,
    State(config): State<EnvironmentConfig>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Intentar extraer token del header Authorization
    if let Some(auth_header) = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_str| auth_str.to_str().ok())
        .and_then(|auth_str| auth_str.strip_prefix("Bearer "))
    {
        // Si hay token, validarlo
        if let Ok(token_data) = decode::<Claims>(
            auth_header,
            &DecodingKey::from_secret(config.jwt_secret.as_ref()),
            &Validation::default(),
        ) {
            let claims = token_data.claims;

            // Verificar que el usuario existe
            if let Ok(user_row) = sqlx::query!(
                r#"
                SELECT 
                    id, company_id, password_hash, full_name, email, created_at
                FROM users 
                WHERE id = $1 
                AND company_id = $2
                "#,
                Uuid::parse_str(&claims.sub).unwrap_or_default(),
                Uuid::parse_str(&claims.company_id).unwrap_or_default()
            )
            .fetch_optional(&pool)
            .await
            {
                if let Some(user_row) = user_row {
                    let authenticated_user = AuthenticatedUser {
                        user_id: user_row.id,
                        company_id: user_row.company_id,
                    };
                    request.extensions_mut().insert(authenticated_user);
                }
            }
        }
    }

    Ok(next.run(request).await)
}

/// Middleware para verificar que el usuario pertenece a la empresa
pub async fn company_access_middleware(
    Extension(_user): Extension<AuthenticatedUser>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Este middleware se puede usar para verificar acceso específico a recursos de la empresa
    // Por ahora solo pasa la request, pero se puede extender para verificaciones específicas
    
    Ok(next.run(request).await)
}

/// Función para generar JWT token simplificado
pub fn generate_jwt_token(
    user_id: Uuid,
    company_id: Uuid,
    config: &EnvironmentConfig,
) -> Result<String, AppError> {
    let now = chrono::Utc::now();
    let expires_at = now + chrono::Duration::seconds(config.jwt_expiration as i64);

    let claims = Claims {
        sub: user_id.to_string(),
        company_id: company_id.to_string(),
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let encoding_key = jsonwebtoken::EncodingKey::from_secret(config.jwt_secret.as_ref());
    
    jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &encoding_key)
        .map_err(|e| AppError::Internal(format!("Error generando JWT: {}", e)))
}