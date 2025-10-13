//! Middleware de CORS
//! 
//! Este módulo maneja la configuración de CORS para permitir
//! requests desde diferentes orígenes.

use axum::http::{HeaderName, HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};

/// Crear middleware de CORS configurado
pub fn cors_middleware() -> CorsLayer {
    // Orígenes específicos para desarrollo y producción
    let allowed_origins = [
        "http://localhost:8080".parse::<HeaderValue>().unwrap(),
        "http://localhost:8081".parse::<HeaderValue>().unwrap(),
        "http://127.0.0.1:8080".parse::<HeaderValue>().unwrap(),
        "http://127.0.0.1:8081".parse::<HeaderValue>().unwrap(),
        "https://api.delivery.nexuslabs.one".parse::<HeaderValue>().unwrap(),
    ];

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
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
        .allow_credentials(false)
        .max_age(std::time::Duration::from_secs(3600))
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
