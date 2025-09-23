//! Modelo de User
//! 
//! Este módulo contiene el struct User y sus variantes para CRUD operations.
//! Mapea exactamente al schema PostgreSQL con primary key 'id'.

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use validator::Validate;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Tipo de usuario - mapea al ENUM user_type
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "user_type", rename_all = "lowercase")]
pub enum UserType {
    Admin,
    Driver,
}

/// Estado del usuario - mapea al ENUM user_status
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
}

/// User principal - mapea exactamente a la tabla users del schema
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub company_id: Uuid,
    pub user_type: UserType,
    pub user_status: UserStatus,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Request para crear un nuevo usuario
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    
    #[validate(length(min = 6, max = 100))]
    pub password: String,
    
    #[validate(email)]
    pub email: String,
    
    pub user_type: UserType,
}

/// Request para actualizar un usuario existente
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,
    
    #[validate(email)]
    pub email: Option<String>,
    
    pub user_type: Option<UserType>,
    pub user_status: Option<UserStatus>,
    
    #[validate(length(min = 6, max = 100))]
    pub password: Option<String>,
}

/// Response de usuario para la API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub company_id: Uuid,
    pub user_type: UserType,
    pub user_status: UserStatus,
    pub username: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response de usuario para listados paginados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

/// Filtros para búsqueda de usuarios
#[derive(Debug, Clone, Deserialize)]
pub struct UserFilters {
    pub username: Option<String>,
    pub email: Option<String>,
    pub user_type: Option<UserType>,
    pub user_status: Option<UserStatus>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub page: Option<i32>,
}

/// Request para login de usuario
#[derive(Debug, Deserialize, Validate)]
pub struct UserLoginRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    
    #[validate(length(min = 6, max = 100))]
    pub password: String,
}

/// Response para login exitoso
#[derive(Debug, Serialize)]
pub struct UserLoginResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            company_id: user.company_id,
            user_type: user.user_type,
            user_status: user.user_status,
            username: user.username,
            email: None, // Campo no existe en el schema actual
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
