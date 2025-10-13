use serde::{Deserialize, Serialize};

// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub token: Option<String>,
    pub message: Option<String>,
    pub company_id: Option<String>,
    pub company_name: Option<String>,
}

impl LoginResponse {
    pub fn success(token: String, company_id: String, company_name: String) -> Self {
        Self {
            success: true,
            token: Some(token),
            message: None,
            company_id: Some(company_id),
            company_name: Some(company_name),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            token: None,
            message: Some(message),
            company_id: None,
            company_name: None,
        }
    }
}

