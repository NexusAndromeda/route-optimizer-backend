mod api;
mod config;
mod state;
mod database;
mod services;
mod utils;
mod clients;
mod models;
mod cache;
mod middleware;
mod analysis;
mod controllers;
mod repositories;
mod routes;
mod dto;

use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
    response::Json,
};
use std::net::SocketAddr;
use tokio::signal;
use tracing::{info, error};
use dotenvy::dotenv;
use serde_json::json;

use config::environment::EnvironmentConfig;
use state::*;
use database::DatabaseConnection;
use middleware::cors::cors_middleware;

use cache::redis_client::RedisClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Cargar variables de entorno
    dotenv().ok();

    // Configurar logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("🚚 Delivery Route Optimizer - API Web Colis Privé");
    info!("================================================");

    // Inicializar base de datos
    let db_connection = match DatabaseConnection::new_default().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("❌ Error conectando a la base de datos: {}", e);
            return Err(anyhow::anyhow!("Error de base de datos: {}", e));
        }
    };
    
    let pool = db_connection.pool().clone();

    // Inicializar Redis y cache
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let redis_config = cache::CacheConfig {
        redis_url,
        default_ttl: 3600,
        max_connections: 10,
    };
    
    let redis_client = match RedisClient::new(redis_config).await {
        Ok(client) => {
            info!("✅ Redis conectado exitosamente");
            client
        }
        Err(e) => {
            error!("❌ Error conectando a Redis: {}", e);
            return Err(anyhow::anyhow!("Error de Redis: {}", e));
        }
    };

    // Crear router de la API
    let app_state = AppState::new(pool, EnvironmentConfig::default(), redis_client);
    
    let app = Router::new()
        .route("/test", get(test_endpoint))
        // Legacy Colis Privé routes (mantener por compatibilidad)
        .route("/colis-prive/companies", get(api::colis_prive::get_companies))
        .route("/colis-prive/packages-test", get(api::colis_prive::test_packages_endpoint))
        .route("/colis-prive/packages", post(api::colis_prive::get_packages))
        // Nuevas rutas MVC
        .nest("/api/company", routes::company_routes::create_company_router())
        .nest("/api/vehicle", routes::vehicle_routes::create_vehicle_router())
        .nest("/api/address", routes::address_routes::create_address_router())
        // migration endpoints eliminados - código legacy
        .merge(api::create_api_router())
        .layer(cors_middleware())
        .with_state(app_state);

    // Puerto del servidor
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;

    info!("🌐 Servidor iniciando en http://{}", addr);
    info!("🔍 Endpoints disponibles:");
    info!("   GET  /test - Endpoint de prueba");
    info!("🏢 Endpoints MVC - Company:");
    info!("   POST /api/company/register - Registrar empresa");
    info!("   POST /api/company/login - Login empresa");
    info!("   GET  /api/company/me - Obtener empresa actual");
    info!("🚗 Endpoints MVC - Vehicle:");
    info!("   POST /api/vehicle - Crear vehículo");
    info!("   GET  /api/vehicle - Listar vehículos");
    info!("   GET  /api/vehicle/:id - Obtener vehículo");
    info!("   PUT  /api/vehicle/:id - Actualizar vehículo");
    info!("   DELETE /api/vehicle/:id - Eliminar vehículo");
    info!("📍 Endpoints MVC - Address:");
    info!("   POST /api/address - Guardar dirección");
    info!("   GET  /api/address/search - Buscar direcciones");
    info!("   GET  /api/address/:id - Obtener dirección");
    info!("   PUT  /api/address/:id - Actualizar código/BAL");
    info!("   DELETE /api/address/:id - Eliminar dirección");
    info!("   GET  /api/address/route/:route_id - Direcciones por ruta");
    info!("📦 Endpoints Legacy - Colis Privé:");
    info!("   GET  colis-prive/health - Health check");
    info!("   GET  colis-prive/companies - Obtener empresas");
    info!("   POST colis-prive/auth - Autenticación");
    info!("   GET  colis-prive/packages-test - Test endpoint");
    info!("   POST colis-prive/packages - Obtener paquetes");
    info!("   POST colis-prive/tournee - Tournée Colis Privé (API Web)");
    // migration endpoints eliminados - código legacy
    info!("🚀 Endpoints Híbridos (Nuevos):");
    info!("   POST hybrid/process - Procesamiento híbrido de paquetes");
    info!("   POST hybrid/package-detail - Obtener datos detallados");
    info!("   POST hybrid/cache/cleanup - Limpiar cache");
    info!("   POST hybrid/cache/stats - Estadísticas de cache");
    info!("📱 Endpoints Móviles (Nuevos):");
    info!("   POST mobile/tournee - Obtener tournée para móvil");
    info!("   POST mobile/package/update-status - Actualizar estado paquete");
    info!("   POST mobile/route/optimize - Optimizar ruta");
    info!("   GET  mobile/stats - Estadísticas móviles");
    info!("🔄 Endpoints de Actualización:");
    info!("   POST mobile/check-update - Verificar actualizaciones disponibles");
    info!("   GET  mobile/download/apk - Descargar APK de la aplicación");
    info!("   GET  mobile/test-download - Endpoint de prueba de descarga");
    info!("   GET  mobile/server-version - Información de versión del servidor");

    // Iniciar servidor en background
    let server_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .map_err(|e| {
                error!("❌ Error del servidor: {}", e);
                e
            })
    });

    // Esperar a que el servidor termine
    if let Err(e) = server_handle.await? {
        error!("❌ Servidor terminó con error: {}", e);
    }

    info!("👋 Servidor terminado");
    Ok(())
}

/// Endpoint de prueba simple
async fn test_endpoint() -> Json<serde_json::Value> {
    Json(json!({
        "message": "¡API Web Colis Privé funcionando correctamente!",
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "api_type": "web_only"
    }))
}

/// Señal de apagado graceful
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("🛑 Señal Ctrl+C recibida, apagando servidor...");
        },
        _ = terminate => {
            info!("🛑 Señal de terminación recibida, apagando servidor...");
        },
    }
}
