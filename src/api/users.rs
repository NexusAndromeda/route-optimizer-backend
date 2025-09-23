//! Handlers de usuarios
//! 
//! Este módulo maneja las operaciones CRUD para usuarios.

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use bcrypt::{hash, DEFAULT_COST};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::user::{User, UserResponse, CreateUserRequest, UserType, UserStatus},
    utils::errors::{AppError, AppResult},
    middleware::auth::AuthenticatedUser,
};

/// Handler para listar usuarios
pub async fn get_users(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
) -> AppResult<Json<Vec<UserResponse>>> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            id, company_id, user_type as "user_type: String", user_status as "user_status: String", 
            username, password_hash, created_at, updated_at, deleted_at
        FROM users 
        WHERE company_id = $1 
        AND deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT 50
        "#,
        user.company_id
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let users: Vec<UserResponse> = rows
        .into_iter()
        .map(|row| {
            UserResponse {
                id: row.id,
                company_id: row.company_id,
                user_type: match row.user_type.as_str() {
                    "admin" => UserType::Admin,
                    "driver" => UserType::Driver,
                    _ => UserType::Driver,
                },
                user_status: match row.user_status.as_str() {
                    "active" => UserStatus::Active,
                    "inactive" => UserStatus::Inactive,
                    "suspended" => UserStatus::Suspended,
                    _ => UserStatus::Active,
                },
                username: row.username,
                email: None,
                created_at: row.created_at.unwrap_or_else(|| chrono::Utc::now()),
                updated_at: row.updated_at.unwrap_or_else(|| chrono::Utc::now()),
            }
        })
        .collect();

    Ok(Json(users))
}

/// Handler para crear usuario
pub async fn create_user(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Json(user_data): Json<CreateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    // Validar datos de entrada
    user_data.validate()
        .map_err(AppError::Validation)?;

    // Hash del password
    let password_hash = hash(&user_data.password, DEFAULT_COST)
        .map_err(|e| AppError::Hash(format!("Error hasheando password: {}", e)))?;

    // Crear usuario
    let row = sqlx::query!(
        r#"
        INSERT INTO users (
            company_id, user_type, user_status, username, 
            password_hash, created_at, updated_at
        ) VALUES ($1, ($2::text)::user_type, ($3::text)::user_status, $4, $5, NOW(), NOW())
        RETURNING 
            id, company_id, user_type as "user_type: crate::models::user::UserType", user_status as "user_status: crate::models::user::UserStatus", 
            username, password_hash, created_at, updated_at, deleted_at
        "#,
        user.company_id,
        match user_data.user_type {
            UserType::Admin => "admin",
            UserType::Driver => "driver",
        },
        "active",
        user_data.username,
        password_hash
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let new_user_response = UserResponse {
        id: row.id,
        company_id: row.company_id,
        user_type: row.user_type,
        user_status: row.user_status,
        username: row.username,
        email: None,
        created_at: row.created_at.unwrap_or_else(|| chrono::Utc::now()),
        updated_at: row.updated_at.unwrap_or_else(|| chrono::Utc::now()),
    };

    Ok(Json(new_user_response))
}

/// Handler para obtener usuario por ID
pub async fn get_user(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    let row = sqlx::query!(
        r#"
        SELECT 
            id, company_id, user_type as "user_type: crate::models::user::UserType", user_status as "user_status: crate::models::user::UserStatus", 
            username, password_hash, created_at, updated_at, deleted_at
        FROM users 
        WHERE id = $1 
        AND company_id = $2
        AND deleted_at IS NULL
        "#,
        user_id,
        user.company_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?
    .ok_or_else(|| AppError::NotFound("Usuario no encontrado".to_string()))?;

    let user_response = UserResponse {
        id: row.id,
        company_id: row.company_id,
        user_type: row.user_type,
        user_status: row.user_status,
        username: row.username,
        email: None,
        created_at: row.created_at.unwrap_or_else(|| chrono::Utc::now()),
        updated_at: row.updated_at.unwrap_or_else(|| chrono::Utc::now()),
    };

    Ok(Json(user_response))
}

/// Handler para actualizar usuario (versión simplificada)
pub async fn update_user(
    Extension(_user): Extension<AuthenticatedUser>,
    State(_state): State<crate::state::AppState>,
    Path(_user_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // Por ahora retorna Not Implemented
    Err(AppError::NotImplemented("Update user no implementado aún".to_string()))
}

/// Handler para eliminar usuario (soft delete)
pub async fn delete_user(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Path(user_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let result = sqlx::query!(
        r#"
        UPDATE users 
        SET deleted_at = NOW(), updated_at = NOW()
        WHERE id = $1 AND company_id = $2 AND deleted_at IS NULL
        "#,
        user_id,
        user.company_id
    )
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Usuario no encontrado".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

impl From<&str> for UserType {
    fn from(s: &str) -> Self {
        match s {
            "admin" => UserType::Admin,
            "driver" => UserType::Driver,
            _ => UserType::Driver,
        }
    }
}
