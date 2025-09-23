use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn test_health_check() {
    let app = create_test_app().await;
    let response = app.get("/api/colis-prive/health").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let body: serde_json::Value = response.json();
    assert_eq!(body["service"], "colis-prive");
    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
async fn test_auth_endpoint_invalid_credentials() {
    let app = create_test_app().await;
    let response = app
        .post("/api/colis-prive/auth")
        .json(&json!({
            "username": "invalid_user",
            "password": "invalid_password",
            "societe": "INVALID_SOCIETE"
        }))
        .await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let body: serde_json::Value = response.json();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "AUTH_FAILED");
}

#[tokio::test]
async fn test_tournee_endpoint_invalid_credentials() {
    let app = create_test_app().await;
    let response = app
        .post("/api/colis-prive/tournee")
        .json(&json!({
            "username": "invalid_user",
            "password": "invalid_password",
            "societe": "INVALID_SOCIETE",
            "date": "2025-08-18",
            "matricule": "INVALID_MATRICULE"
        }))
        .await;
    
    // Debería fallar pero no dar error 500
    assert_ne!(response.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

// Test eliminado: mobile-tournee endpoint no está implementado (solo web API)

#[tokio::test]
async fn test_endpoint_comparison() {
    let app = create_test_app().await;
    
    // Test data común
    let test_data = json!({
        "username": "test_user",
        "password": "test_password",
        "societe": "TEST_SOCIETE",
        "date": "2025-08-18",
        "matricule": "TEST_SOCIETE_TEST_USER"
    });
    
    // Probar endpoint web
    let web_response = app
        .post("/api/colis-prive/tournee")
        .json(&test_data)
        .await;
    
    // Solo probar endpoint web (mobile endpoint eliminado)
    assert_ne!(web_response.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    
    // Verificar que el endpoint web tenga la estructura correcta
    let web_body: serde_json::Value = web_response.json();
    assert!(web_body.is_object());
}

// Función helper para crear la app de test
async fn create_test_app() -> axum::Router {
    // Crear una app de test básica
    axum::Router::new()
        .route("/api/colis-prive/health", axum::routing::get(|| async { "OK" }))
        .route("/api/colis-prive/auth", axum::routing::post(|| async { "OK" }))
        .route("/api/colis-prive/tournee", axum::routing::post(|| async { "OK" }))
        // mobile-tournee endpoint eliminado (solo web API)
}

