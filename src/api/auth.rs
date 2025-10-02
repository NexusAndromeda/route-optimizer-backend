//! Handlers de autenticación
//! 
//! Este módulo maneja el login, registro y renovación de tokens JWT.

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    config::EnvironmentConfig,
    models::user::{User, UserType, UserStatus, UserResponse},
    utils::errors::{AppError, AppResult},
    utils::jwt::{generate_token, JwtConfig},
    middleware::auth::AuthenticatedUser,
    state::AppState,
};

/// Request de login
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    
    #[validate(length(min = 6, max = 100))]
    pub password: String,
}

/// Response de login exitoso
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: UserResponse,
}

/// Response de login optimizado para Android con token en múltiples ubicaciones
#[derive(Debug, Serialize)]
pub struct LoginResponseFlexible {
    pub success: bool,
    pub status: String,           // "200"
    pub code: String,            // "200" 
    pub token: String,           // Token directo en raíz
    pub message: String,         // Mensaje directo en raíz
    pub authentication: Option<AuthInfo>,
    pub credentials_used: Option<CredentialsUsed>,
    pub timestamp: String,
}

/// Información de autenticación
#[derive(Debug, Serialize)]
pub struct AuthInfo {
    pub token: String,
    pub matricule: String,
    pub message: String,
}

/// Credenciales utilizadas
#[derive(Debug, Serialize)]
pub struct CredentialsUsed {
    pub username: String,
    pub timestamp: String,
}

/// Request para refresh token
#[derive(Debug, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 10))]
    pub refresh_token: String,
}

/// Response de refresh token
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

/// Request de registro
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 6, max = 100))]
    pub password: String,
    
    pub user_type: UserType,
}

/// Handler de login
pub async fn login(
    State(app_state): State<AppState>,
    Json(login_data): Json<LoginRequest>,
) -> AppResult<Json<LoginResponseFlexible>> {
    let pool = &app_state.pool;
    let config = &app_state.config;
    // Validar datos de entrada
    login_data.validate()
        .map_err(AppError::Validation)?;

    // Buscar usuario por username
    let row = sqlx::query!(
        r#"
        SELECT 
            id, company_id, user_type as "user_type: String", user_status as "user_status: String", username, 
            password_hash, created_at, updated_at, deleted_at
        FROM users 
        WHERE username = $1 
        AND deleted_at IS NULL
        "#,
        login_data.username
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e))?
    .ok_or_else(|| AppError::Unauthorized("Credenciales inválidas".to_string()))?;

    let user = User {
        id: row.id,
        company_id: row.company_id,
        user_type: match row.user_type.as_str() {
            "admin" => UserType::Admin,
            "driver" => UserType::Driver,
            _ => return Err(AppError::Database(sqlx::Error::Decode("Invalid user_type".into()))),
        },
        user_status: match row.user_status.as_str() {
            "active" => UserStatus::Active,
            "inactive" => UserStatus::Inactive,
            "suspended" => UserStatus::Suspended,
            _ => return Err(AppError::Database(sqlx::Error::Decode("Invalid user_status".into()))),
        },
        username: row.username,
        password_hash: row.password_hash,
        created_at: row.created_at.expect("created_at should not be null"),
        updated_at: row.updated_at.expect("updated_at should not be null"),
        deleted_at: row.deleted_at,
    };

    // Verificar que el usuario esté activo
    if user.user_status != UserStatus::Active {
        return Err(AppError::Unauthorized("Usuario inactivo o suspendido".to_string()));
    }

    // Verificar password
    let password_valid = verify(&login_data.password, &user.password_hash)
        .map_err(|e| AppError::Hash(format!("Error verificando password: {}", e)))?;

    if !password_valid {
        return Err(AppError::Unauthorized("Credenciales inválidas".to_string()));
    }

    // Generar JWT token
    let jwt_config = JwtConfig::from(config);
    let access_token = generate_token(user.id, user.company_id, user.user_type.clone(), &jwt_config)?;

    // Convertir a response
    let user_response = UserResponse::from(user);

    // Respuesta MÁS COMPATIBLE para Android
    let response = LoginResponseFlexible {
        success: true,
        status: "200".to_string(),
        code: "200".to_string(),
        token: access_token.clone(),      // TOKEN EN RAÍZ
        message: "Login exitoso".to_string(),  // MESSAGE EN RAÍZ
        authentication: Some(AuthInfo {
            token: access_token.clone(),   // TOKEN TAMBIÉN EN AUTH
            matricule: user_response.username.clone(),
            message: "Autenticación exitosa".to_string(),
        }),
        credentials_used: Some(CredentialsUsed {
            username: login_data.username.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(response))
}

/// Handler de registro
pub async fn register(
    State(app_state): State<AppState>,
    Json(register_data): Json<RegisterRequest>,
) -> AppResult<Json<LoginResponseFlexible>> {
    let pool = &app_state.pool;
    let config = &app_state.config;
    // Validar datos de entrada
    register_data.validate()
        .map_err(AppError::Validation)?;

    // Verificar que el username no exista
    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE username = $1 AND deleted_at IS NULL",
        register_data.username
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    if existing_user.is_some() {
        return Err(AppError::Conflict("Username ya existe".to_string()));
    }

    // Verificar que el email no exista
    let existing_email = sqlx::query!(
        "SELECT id FROM users WHERE email = $1 AND deleted_at IS NULL",
        register_data.email
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    if existing_email.is_some() {
        return Err(AppError::Conflict("Email ya existe".to_string()));
    }

    // Hash del password
    let password_hash = hash(&register_data.password, DEFAULT_COST)
        .map_err(|e| AppError::Hash(format!("Error hasheando password: {}", e)))?;

    // Crear usuario (asumiendo que se crea en una empresa por defecto)
    // En producción, esto debería ser manejado por un admin o proceso de onboarding
    let company_id = Uuid::new_v4(); // Placeholder - en producción esto vendría del contexto

    let row = sqlx::query!(
        r#"
        INSERT INTO users (
            company_id, user_type, user_status, username, 
            password_hash, created_at, updated_at
        ) VALUES ($1, ($2::text)::user_type, ($3::text)::user_status, $4, $5, NOW(), NOW())
        RETURNING 
            id, company_id, user_type as "user_type: crate::models::user::UserType", user_status as "user_status: crate::models::user::UserStatus", username, 
            password_hash, created_at, updated_at, deleted_at
        "#,
        company_id,
        match register_data.user_type {
            UserType::Admin => "admin",
            UserType::Driver => "driver",
        },
        "active",
        register_data.username,
        password_hash
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let new_user = User {
        id: row.id,
        company_id: row.company_id,
        user_type: row.user_type,
        user_status: row.user_status,
        username: row.username,
        password_hash: row.password_hash,
        created_at: row.created_at.expect("created_at should not be null"),
        updated_at: row.updated_at.expect("updated_at should not be null"),
        deleted_at: row.deleted_at,
    };

    // Generar JWT token
    let jwt_config = JwtConfig::from(config);
    let access_token = generate_token(new_user.id, new_user.company_id, new_user.user_type.clone(), &jwt_config)?;

    // Convertir a response
    let user_response = UserResponse::from(new_user);

    // Respuesta MÁS COMPATIBLE para Android
    let response = LoginResponseFlexible {
        success: true,
        status: "200".to_string(),
        code: "200".to_string(),
        token: access_token.clone(),      // TOKEN EN RAÍZ
        message: "Registro exitoso".to_string(),  // MESSAGE EN RAÍZ
        authentication: Some(AuthInfo {
            token: access_token.clone(),   // TOKEN TAMBIÉN EN AUTH
            matricule: user_response.username.clone(),
            message: "Usuario registrado exitosamente".to_string(),
        }),
        credentials_used: Some(CredentialsUsed {
            username: register_data.username.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(response))
}

/// Handler para obtener información del usuario autenticado
pub async fn me(
    Extension(user): Extension<AuthenticatedUser>,
    State(app_state): State<AppState>,
) -> AppResult<Json<UserResponse>> {
    let pool = &app_state.pool;
    // Buscar usuario completo
    let row = sqlx::query!(
        r#"
        SELECT 
            id, company_id, user_type as "user_type: crate::models::user::UserType", user_status as "user_status: crate::models::user::UserStatus", username, 
            password_hash, created_at, updated_at, deleted_at
        FROM users 
        WHERE id = $1 
        AND deleted_at IS NULL
        "#,
        user.user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let user_data = User {
        id: row.id,
        company_id: row.company_id,
        user_type: row.user_type,
        user_status: row.user_status,
        username: row.username,
        password_hash: row.password_hash,
        created_at: row.created_at.expect("created_at should not be null"),
        updated_at: row.updated_at.expect("updated_at should not be null"),
        deleted_at: row.deleted_at,
    };

    let user_response = UserResponse::from(user_data);
    Ok(Json(user_response))
}

/// Handler de refresh token
pub async fn refresh_token(
    State(_app_state): State<AppState>,
    Json(refresh_data): Json<RefreshTokenRequest>,
) -> AppResult<Json<RefreshTokenResponse>> {
    // Validar datos de entrada
    refresh_data.validate()
        .map_err(AppError::Validation)?;

    // En una implementación real, aquí verificarías el refresh token
    // y generarías un nuevo access token
    // Por ahora, retornamos un error indicando que no está implementado
    
    Err(AppError::Unauthorized("Refresh token no implementado aún".to_string()))
}

/// Handler de logout
pub async fn logout() -> StatusCode {
    // En una implementación real, aquí invalidarías el token
    // Por ahora, solo retornamos OK
    StatusCode::OK
}

use axum::Router;

/// Crear el router de autenticación
pub fn create_auth_router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/login", axum::routing::post(login))
        .route("/register", axum::routing::post(register))
        .route("/me", axum::routing::get(me))
        .route("/refresh", axum::routing::post(refresh_token))
        .route("/logout", axum::routing::post(logout))
}
