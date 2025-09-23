//! Sistema de manejo de errores
//! 
//! Este módulo define todos los tipos de errores del sistema
//! y su conversión a respuestas HTTP apropiadas.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Errores principales de la aplicación
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("JWT error: {0}")]
    Jwt(String),

    #[error("Hash error: {0}")]
    Hash(String),

    #[error("External API error: {0}")]
    ExternalApi(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

/// Respuesta de error para la API
#[derive(Debug, serde::Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            AppError::Database(e) => {
                eprintln!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        error: "Database Error".to_string(),
                        message: "An error occurred while accessing the database".to_string(),
                        details: Some(json!({ "sql_error": e.to_string() })),
                        code: Some("DB_ERROR".to_string()),
                    },
                )
            }

            AppError::Validation(e) => {
                eprintln!("Validation error: {}", e);
                (
                    StatusCode::BAD_REQUEST,
                    ErrorResponse {
                        error: "Validation Error".to_string(),
                        message: "The provided data is invalid".to_string(),
                        details: Some(json!(e)),
                        code: Some("VALIDATION_ERROR".to_string()),
                    },
                )
            }

            AppError::Unauthorized(msg) => {
                eprintln!("Unauthorized access: {}", msg);
                (
                    StatusCode::UNAUTHORIZED,
                    ErrorResponse {
                        error: "Unauthorized".to_string(),
                        message: msg,
                        details: None,
                        code: Some("UNAUTHORIZED".to_string()),
                    },
                )
            }

            AppError::Forbidden(msg) => {
                eprintln!("Forbidden access: {}", msg);
                (
                    StatusCode::FORBIDDEN,
                    ErrorResponse {
                        error: "Forbidden".to_string(),
                        message: msg,
                        details: None,
                        code: Some("FORBIDDEN".to_string()),
                    },
                )
            }

            AppError::NotFound(msg) => {
                eprintln!("Resource not found: {}", msg);
                (
                    StatusCode::NOT_FOUND,
                    ErrorResponse {
                        error: "Not Found".to_string(),
                        message: msg,
                        details: None,
                        code: Some("NOT_FOUND".to_string()),
                    },
                )
            }

            AppError::Conflict(msg) => {
                eprintln!("Conflict: {}", msg);
                (
                    StatusCode::CONFLICT,
                    ErrorResponse {
                        error: "Conflict".to_string(),
                        message: msg,
                        details: None,
                        code: Some("CONFLICT".to_string()),
                    },
                )
            }

            AppError::BadRequest(msg) => {
                eprintln!("Bad request: {}", msg);
                (
                    StatusCode::BAD_REQUEST,
                    ErrorResponse {
                        error: "Bad Request".to_string(),
                        message: msg,
                        details: None,
                        code: Some("BAD_REQUEST".to_string()),
                    },
                )
            }

            AppError::Internal(msg) => {
                eprintln!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        error: "Internal Server Error".to_string(),
                        message: "An unexpected error occurred".to_string(),
                        details: Some(json!({ "internal_error": msg })),
                        code: Some("INTERNAL_ERROR".to_string()),
                    },
                )
            }

            AppError::RateLimitExceeded => {
                eprintln!("Rate limit exceeded");
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    ErrorResponse {
                        error: "Rate Limit Exceeded".to_string(),
                        message: "Too many requests. Please try again later".to_string(),
                        details: None,
                        code: Some("RATE_LIMIT_EXCEEDED".to_string()),
                    },
                )
            }

            AppError::ServiceUnavailable(msg) => {
                eprintln!("Service unavailable: {}", msg);
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    ErrorResponse {
                        error: "Service Unavailable".to_string(),
                        message: msg,
                        details: None,
                        code: Some("SERVICE_UNAVAILABLE".to_string()),
                    },
                )
            }

            AppError::Jwt(msg) => {
                eprintln!("JWT error: {}", msg);
                (
                    StatusCode::UNAUTHORIZED,
                    ErrorResponse {
                        error: "JWT Error".to_string(),
                        message: msg,
                        details: None,
                        code: Some("JWT_ERROR".to_string()),
                    },
                )
            }

            AppError::Hash(msg) => {
                eprintln!("Hash error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        error: "Hash Error".to_string(),
                        message: "An error occurred while processing credentials".to_string(),
                        details: Some(json!({ "hash_error": msg })),
                        code: Some("HASH_ERROR".to_string()),
                    },
                )
            }

            AppError::ExternalApi(msg) => {
                eprintln!("External API error: {}", msg);
                (
                    StatusCode::BAD_GATEWAY,
                    ErrorResponse {
                        error: "External API Error".to_string(),
                        message: "An error occurred while communicating with external service".to_string(),
                        details: Some(json!({ "external_api_error": msg })),
                        code: Some("EXTERNAL_API_ERROR".to_string()),
                    },
                )
            }

            AppError::NotImplemented(msg) => {
                eprintln!("Not implemented: {}", msg);
                (
                    StatusCode::NOT_IMPLEMENTED,
                    ErrorResponse {
                        error: "Not Implemented".to_string(),
                        message: msg,
                        details: None,
                        code: Some("NOT_IMPLEMENTED".to_string()),
                    },
                )
            }
        };

        (status, Json(error_response)).into_response()
    }
}

/// Resultado tipado para operaciones que pueden fallar
pub type AppResult<T> = Result<T, AppError>;

/// Función helper para crear errores de validación
pub fn validation_error(field: &'static str, message: &'static str) -> AppError {
    use validator::ValidationError;
    
    let mut error = ValidationError::new("custom");
    error.add_param("field".into(), &field);
    error.add_param("message".into(), &message);
    
    let mut errors = validator::ValidationErrors::new();
    errors.add(field, error);
    
    AppError::Validation(errors)
}

/// Función helper para crear errores de recurso no encontrado
pub fn not_found_error(resource: &str, id: &str) -> AppError {
    AppError::NotFound(format!("{} with id '{}' not found", resource, id))
}

/// Función helper para crear errores de conflicto
pub fn conflict_error(resource: &str, field: &str, value: &str) -> AppError {
    AppError::Conflict(format!("{} with {} '{}' already exists", resource, field, value))
}

/// Función helper para crear errores de acceso prohibido
pub fn forbidden_error(operation: &str, reason: &str) -> AppError {
    AppError::Forbidden(format!("Cannot {}: {}", operation, reason))
}

/// Función helper para crear errores de solicitud incorrecta
pub fn bad_request_error(message: &str) -> AppError {
    AppError::BadRequest(message.to_string())
}

/// Función helper para crear errores internos
pub fn internal_error(message: &str) -> AppError {
    AppError::Internal(message.to_string())
}
