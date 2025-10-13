use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Request para registrar una empresa
#[derive(Debug, Deserialize)]
pub struct RegisterCompanyRequest {
    pub company_name: String,
    pub company_address: String,
    pub company_siret: Option<String>,
    pub admin_full_name: String,
    pub admin_email: String,
    pub admin_password: String,
}

// Response de empresa (sin password)
#[derive(Debug, Serialize)]
pub struct CompanyResponse {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub siret: Option<String>,
    pub admin_full_name: String,
    pub admin_email: String,
    pub subscription_plan: String,
    pub subscription_status: String,
    pub created_at: DateTime<Utc>,
}

// Response genérica
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            message: None,
            data: Some(data),
        }
    }

    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            message: Some(message),
            data: Some(data),
        }
    }
}

impl ApiResponse<()> {
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message: Some(message),
            data: None,
        }
    }
}

