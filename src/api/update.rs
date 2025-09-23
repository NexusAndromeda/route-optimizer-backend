use axum::{extract::Json, response::Json as AxumJson, response::Response, http::StatusCode, body::Body, extract::State};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use sqlx::Row;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppVersion {
    pub version_name: String,
    pub version_code: i32,
    pub build_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateData {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
    pub force_update: bool,
    pub min_supported_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCheckResponse {
    pub success: bool,
    pub data: Option<UpdateData>,
    pub error: Option<String>,
}

/// Verificar si hay actualizaciones disponibles
pub async fn check_for_updates(
    State(state): State<AppState>,
    Json(version): Json<AppVersion>
) -> AxumJson<serde_json::Value> {
    log::info!("🔄 Verificando actualizaciones para versión: {}", version.version_name);
    
    let current_version = format!("{}.{}", version.version_name, version.version_code);
    
    // Consultar la base de datos para obtener la versión más reciente
    match sqlx::query(
        r#"
        SELECT 
            version_name,
            version_code,
            download_url,
            release_notes,
            release_notes_fr,
            release_notes_en,
            force_update,
            min_supported_version,
            apk_size_bytes,
            is_active,
            is_beta
        FROM app_versions 
        WHERE is_active = TRUE 
        ORDER BY version_code DESC 
        LIMIT 1
        "#
    )
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => {
            let latest_version_code: i32 = row.get("version_code");
            let latest_version_name: String = row.get("version_name");
            let download_url: String = row.get("download_url");
            let release_notes: Option<String> = row.get("release_notes");
            let force_update: bool = row.get("force_update");
            let min_supported_version: Option<String> = row.get("min_supported_version");
            let apk_size_bytes: Option<i64> = row.get("apk_size_bytes");
            let is_beta: bool = row.get("is_beta");
            
            let latest_version = format!("{}.{}", latest_version_name, latest_version_code);
            
            // Verificar si hay actualización disponible
            let update_available = version.version_code < latest_version_code;
            
            if update_available {
                log::info!("🆕 Actualización disponible: {} -> {}", current_version, latest_version);
                
                let response = json!({
                    "success": true,
                    "data": {
                        "current_version": current_version,
                        "latest_version": latest_version,
                        "update_available": true,
                        "download_url": download_url,
                        "release_notes": release_notes.unwrap_or_else(|| "Nueva versión disponible".to_string()),
                        "force_update": force_update,
                        "min_supported_version": min_supported_version.unwrap_or_else(|| "1.0.0".to_string()),
                        "apk_size_bytes": apk_size_bytes,
                        "is_beta": is_beta
                    },
                    "error": null
                });
                
                AxumJson(response)
            } else {
                log::info!("✅ Aplicación actualizada: {}", current_version);
                
                let response = json!({
                    "success": true,
                    "data": {
                        "current_version": current_version,
                        "latest_version": current_version,
                        "update_available": false,
                        "download_url": null,
                        "release_notes": null,
                        "force_update": false,
                        "min_supported_version": min_supported_version.unwrap_or_else(|| "1.0.0".to_string())
                    },
                    "error": null
                });
                
                AxumJson(response)
            }
        }
        Err(e) => {
            log::error!("❌ Error consultando versiones en la base de datos: {}", e);
            
            let response = json!({
                "success": false,
                "data": null,
                "error": "Error interno del servidor al verificar actualizaciones"
            });
            
            AxumJson(response)
        }
    }
}

/// Obtener información de la versión actual del servidor
pub async fn get_server_version() -> AxumJson<serde_json::Value> {
    let version_info = json!({
        "server_version": "1.0.0",
        "api_version": "1.0.0",
        "build_date": "2024-01-01",
        "environment": "production"
    });
    
    AxumJson(version_info)
}

/// Endpoint de prueba para verificar que el State funciona
pub async fn test_download(State(state): State<AppState>) -> AxumJson<serde_json::Value> {
    log::info!("🧪 Endpoint de prueba de descarga llamado");
    
    let response = json!({
        "message": "Endpoint de prueba funcionando",
        "database_connected": true,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    AxumJson(response)
}

/// Descargar el APK de la aplicación
pub async fn download_apk(State(state): State<AppState>) -> Result<Response<Body>, StatusCode> {
    log::info!("📥 Solicitud de descarga de APK recibida");
    
    // Obtener la versión más reciente de la base de datos
    match sqlx::query(
        r#"
        SELECT 
            apk_path,
            version_name,
            version_code,
            apk_size_bytes,
            apk_checksum
        FROM app_versions 
        WHERE is_active = TRUE 
        ORDER BY version_code DESC 
        LIMIT 1
        "#
    )
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => {
            let apk_path_str: String = row.get("apk_path");
            let version_name: String = row.get("version_name");
            let version_code: i32 = row.get("version_code");
            let apk_size_bytes: Option<i64> = row.get("apk_size_bytes");
            let apk_checksum: Option<String> = row.get("apk_checksum");
            
            let apk_path = Path::new(&apk_path_str);
            
            if !apk_path.exists() {
                log::error!("❌ APK no encontrado en: {:?}", apk_path);
                return Err(StatusCode::NOT_FOUND);
            }
            
            match tokio::fs::read(apk_path).await {
                Ok(apk_data) => {
                    log::info!("✅ APK leído exitosamente: {} v{} ({} bytes)", version_name, version_code, apk_data.len());
                    
                    // Actualizar contador de descargas
                    if let Err(e) = sqlx::query(
                        "UPDATE app_versions SET download_count = download_count + 1, last_downloaded_at = NOW() WHERE version_code = $1"
                    )
                    .bind(version_code)
                    .execute(&state.pool)
                    .await
                    {
                        log::warn!("⚠️ Error actualizando contador de descargas: {}", e);
                    }
                    
                    let filename = format!("delivery-routing-{}.apk", version_name);
                    let mut response_builder = Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "application/vnd.android.package-archive")
                        .header("Content-Disposition", format!("attachment; filename=\"{}\"", filename))
                        .header("Content-Length", apk_data.len().to_string());
                    
                    // Agregar headers adicionales si están disponibles
                    if let Some(size) = apk_size_bytes {
                        response_builder = response_builder.header("X-APK-Size", size.to_string());
                    }
                    
                    if let Some(checksum) = apk_checksum {
                        response_builder = response_builder.header("X-APK-Checksum", checksum);
                    }
                    
                    let response = response_builder
                        .body(Body::from(apk_data))
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                    
                    Ok(response)
                }
                Err(e) => {
                    log::error!("❌ Error leyendo APK: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            log::error!("❌ Error consultando APK en la base de datos: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
