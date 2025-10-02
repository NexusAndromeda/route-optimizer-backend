//! Modelo de User Simplificado
//! 
//! Este módulo contiene el struct User simplificado para el schema simplificado.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// User simplificado - mapea exactamente a la tabla users del schema simplificado
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub company_id: Uuid,
    pub password_hash: String,
    pub full_name: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Request para crear un nuevo usuario
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 2, max = 100))]
    pub full_name: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 6, max = 100))]
    pub password: String,
}

/// Request para actualizar un usuario existente
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 2, max = 100))]
    pub full_name: Option<String>,
    
    #[validate(email)]
    pub email: Option<String>,
    
    #[validate(length(min = 6, max = 100))]
    pub password: Option<String>,
}

/// Response de usuario para la API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub company_id: Uuid,
    pub full_name: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
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
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub page: Option<i32>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            company_id: user.company_id,
            full_name: user.full_name,
            email: user.email,
            created_at: user.created_at,
        }
    }
}