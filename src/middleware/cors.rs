//! Middleware de CORS
//! 
//! Este módulo maneja la configuración de CORS para permitir
//! requests desde diferentes orígenes.

use axum::http::{HeaderName, HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};

/// Crear middleware de CORS configurado para desarrollo
/// NOTA: Permite cualquier origen - solo para desarrollo
pub fn cors_middleware() -> CorsLayer {
    CorsLayer::very_permissive()
}

/// Crear middleware de CORS con orígenes específicos
pub fn cors_middleware_with_origins(origins: Vec<String>) -> CorsLayer {
    let mut cors = CorsLayer::new();
    
    // Agregar orígenes específicos
    for origin in origins {
        if let Ok(header_value) = HeaderValue::from_str(&origin) {
            cors = cors.allow_origin(header_value);
        }
    }
    
    cors.allow_methods([
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::PATCH,
        Method::OPTIONS,
    ])
    .allow_headers([
        HeaderName::from_static("authorization"),
        HeaderName::from_static("content-type"),
        HeaderName::from_static("accept"),
        HeaderName::from_static("origin"),
        HeaderName::from_static("x-requested-with"),
    ])
    .allow_credentials(true)
    .max_age(std::time::Duration::from_secs(3600))
}
