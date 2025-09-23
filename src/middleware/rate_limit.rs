//! Middleware de Rate Limiting
//! 
//! Este módulo maneja la limitación de velocidad de requests
//! para prevenir abuso de la API.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::config::EnvironmentConfig;

/// Estructura para almacenar información de rate limiting por IP
#[derive(Debug, Clone)]
struct RateLimitInfo {
    requests: u32,
    window_start: Instant,
}

/// Estado global del rate limiting
#[derive(Clone)]
pub struct RateLimitState {
    requests: Arc<RwLock<HashMap<String, RateLimitInfo>>>,
    max_requests: u32,
    window_duration: Duration,
}

impl RateLimitState {
    /// Crear nuevo estado de rate limiting
    pub fn new(config: &EnvironmentConfig) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests: config.rate_limit_requests,
            window_duration: Duration::from_secs(config.rate_limit_window as u64),
        }
    }

    /// Verificar si una IP ha excedido el límite
    pub async fn check_rate_limit(&self, ip: &str) -> Result<(), RateLimitError> {
        let mut requests = self.requests.write().await;
        let now = Instant::now();

        // Limpiar entradas expiradas
        requests.retain(|_, info| {
            now.duration_since(info.window_start) < self.window_duration
        });

        // Obtener o crear información de rate limiting para esta IP
        let info = requests.entry(ip.to_string()).or_insert(RateLimitInfo {
            requests: 0,
            window_start: now,
        });

        // Verificar si la ventana de tiempo ha expirado
        if now.duration_since(info.window_start) >= self.window_duration {
            info.requests = 1;
            info.window_start = now;
            return Ok(());
        }

        // Verificar si se ha excedido el límite
        if info.requests >= self.max_requests {
            return Err(RateLimitError::LimitExceeded);
        }

        // Incrementar contador de requests
        info.requests += 1;
        Ok(())
    }
}

/// Errores de rate limiting
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded")]
    LimitExceeded,
}

/// Middleware de rate limiting
pub async fn rate_limit_middleware(
    State(rate_limit_state): State<RateLimitState>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    // Extraer IP del cliente (simplificado - en producción usarías headers reales)
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .split(',')
        .next()
        .unwrap_or("unknown")
        .trim();

    // Verificar rate limit
    if let Err(RateLimitError::LimitExceeded) = rate_limit_state.check_rate_limit(ip).await {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Please try again later.".to_string(),
        ));
    }

    Ok(next.run(request).await)
}

/// Middleware de rate limiting más estricto para endpoints sensibles
pub async fn strict_rate_limit_middleware(
    State(rate_limit_state): State<RateLimitState>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    // Para endpoints sensibles, usar límites más estrictos
    let strict_state = RateLimitState {
        requests: rate_limit_state.requests.clone(),
        max_requests: rate_limit_state.max_requests / 2, // Límite más estricto
        window_duration: rate_limit_state.window_duration,
    };

    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .split(',')
        .next()
        .unwrap_or("unknown")
        .trim();

    if let Err(RateLimitError::LimitExceeded) = strict_state.check_rate_limit(ip).await {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded for sensitive endpoint.".to_string(),
        ));
    }

    Ok(next.run(request).await)
}
