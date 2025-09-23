//! Modelos de autenticación
//! 
//! Este módulo contiene los modelos para login, registro y autenticación JWT.

use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;

/// Request para registro de empresa y admin
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 2, max = 255))]
    pub company_name: String,
    
    #[validate(length(min = 10, max = 500))]
    pub company_address: String,
    
    #[validate(length(min = 3, max = 50))]
    pub admin_username: String,
    
    #[validate(length(min = 8, max = 100))]
    pub admin_password: String,
    
    #[validate(length(min = 2, max = 255))]
    pub admin_full_name: String,
    
    #[validate(email)]
    pub admin_email: String,
    
    #[validate(length(min = 10, max = 20))]
    pub admin_phone: Option<String>,
}

/// Request para login
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 100))]
    pub username: String,
    
    #[validate(length(min = 8, max = 100))]
    pub password: String,
    
    pub company_id: Option<Uuid>,
}

/// Response de autenticación
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: UserResponse,
    pub company: CompanyResponse,
}

/// Response de usuario para autenticación
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub user_type: String,
    pub user_status: String,
    pub tournee_number: Option<String>,
    pub company_id: String,
}

/// Response de empresa para autenticación
#[derive(Debug, Serialize)]
pub struct CompanyResponse {
    pub id: String,
    pub name: String,
    pub subscription_plan: String,
    pub subscription_status: String,
    pub max_drivers: i32,
    pub max_vehicles: i32,
}

/// Request para cambio de contraseña
#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 8, max = 100))]
    pub current_password: String,
    
    #[validate(length(min = 8, max = 100))]
    pub new_password: String,
    
    #[validate(length(min = 8, max = 100))]
    pub confirm_password: String,
}

/// Request para reset de contraseña
#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(email)]
    pub email: String,
}

/// Request para confirmar reset de contraseña
#[derive(Debug, Deserialize, Validate)]
pub struct ConfirmResetPasswordRequest {
    pub reset_token: String,
    
    #[validate(length(min = 8, max = 100))]
    pub new_password: String,
    
    #[validate(length(min = 8, max = 100))]
    pub confirm_password: String,
}
