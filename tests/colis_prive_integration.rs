use crate::client::ColisPriveClient;
use crate::external_models::{LoginRequest, Commun, RefreshTokenRequest, DeviceInfo};
use serde_json::json;

// Device info de prueba para todos los tests
fn create_test_device_info() -> DeviceInfo {
    DeviceInfo {
        model: "Samsung SM-S916B".to_string(),
        imei: "351680012345678".to_string(),
        serial_number: "3qtg83zabc123def456".to_string(),
        android_version: "14".to_string(),
        install_id: "f3a21c95-1a84-4d73-9f65-8e40b7c92ed1".to_string(),
    }
}

#[tokio::test]
async fn test_client_creation() {
    let device_info = create_test_device_info();
    let client = ColisPriveClient::new(device_info);
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_colis_headers() {
    let device_info = create_test_device_info();
    let client = ColisPriveClient::new(device_info).unwrap();
    
    // Test headers para endpoint de login
    let headers = client.get_colis_headers("login", Some("test_user"), None);
    assert!(headers.contains_key("ActivityId"));
    assert!(headers.contains_key("AppName"));
    assert!(headers.contains_key("Device"));
    assert!(headers.contains_key("UserName"));
    assert!(!headers.contains_key("SsoHopps")); // No token en login
    
    // Test headers para endpoint de tournée
    let headers = client.get_colis_headers("tournee", Some("test_user"), Some("test_token"));
    assert!(headers.contains_key("SsoHopps")); // Con token
    assert!(headers.contains_key("X-Requested-With"));
    
    // Test headers para endpoint de refresh
    let headers = client.get_colis_headers("refresh", None, None);
    assert!(!headers.contains_key("UserName")); // Sin username
    assert!(!headers.contains_key("SsoHopps")); // Sin token
}

#[tokio::test]
async fn test_refresh_token_request() {
    let device_info = create_test_device_info();
    let request = RefreshTokenRequest {
        token: "test_token_123".to_string(),
        device_info,
    };
    
    assert_eq!(request.token, "test_token_123");
    assert_eq!(request.device_info.model, "Samsung SM-S916B");
}

#[tokio::test]
async fn test_login_request() {
    let device_info = create_test_device_info();
    let request = LoginRequest {
        login: "test_user".to_string(),
        password: "test_pass".to_string(),
        societe: "test_societe".to_string(),
        commun: Commun {
            duree_token_in_hour: 24,
        },
    };
    
    assert_eq!(request.login, "test_user");
    assert_eq!(request.password, "test_pass");
    assert_eq!(request.societe, "test_societe");
}

#[tokio::test]
async fn test_refresh_token_request_struct() {
    let device_info = create_test_device_info();
    let request = RefreshTokenRequest {
        token: "test_token_456".to_string(),
        device_info: device_info.clone(),
    };
    
    // Serializar a JSON
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("test_token_456"));
    assert!(json.contains("Samsung SM-S916B"));
    
    // Deserializar desde JSON
    let deserialized: RefreshTokenRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.token, "test_token_456");
    assert_eq!(deserialized.device_info.model, "Samsung SM-S916B");
}

#[tokio::test]
async fn test_legacy_headers_compatibility() {
    let device_info = create_test_device_info();
    let client = ColisPriveClient::new(device_info).unwrap();
    
    // Test que get_common_headers() funciona (método legacy)
    let headers = client.get_common_headers();
    assert!(headers.contains_key("Accept-Charset"));
    assert!(headers.contains_key("Content-Type"));
    assert!(headers.contains_key("User-Agent"));
}

#[tokio::test]
async fn test_activity_id_uniqueness() {
    let device_info = create_test_device_info();
    let client = ColisPriveClient::new(device_info).unwrap();
    
    // Generar headers múltiples veces
    let headers1 = client.get_colis_headers("login", Some("user1"), None);
    let headers2 = client.get_colis_headers("login", Some("user2"), None);
    
    let activity_id1 = headers1.get("ActivityId").unwrap().to_str().unwrap();
    let activity_id2 = headers2.get("ActivityId").unwrap().to_str().unwrap();
    
    // Los ActivityId deben ser únicos
    assert_ne!(activity_id1, activity_id2);
}

#[tokio::test]
async fn test_endpoint_specific_headers() {
    let device_info = create_test_device_info();
    let client = ColisPriveClient::new(device_info).unwrap();
    
    // Test headers específicos de login
    let login_headers = client.get_colis_headers("login", Some("user"), None);
    assert!(login_headers.contains_key("Origin"));
    assert!(login_headers.contains_key("Referer"));
    assert!(login_headers.contains_key("Accept-Language"));
    
    // Test headers específicos de tournée
    let tournee_headers = client.get_colis_headers("tournee", Some("user"), Some("token"));
    assert!(tournee_headers.contains_key("X-Requested-With"));
    assert!(tournee_headers.contains_key("X-Device-Info"));
    
    // Test headers específicos de refresh
    let refresh_headers = client.get_colis_headers("refresh", None, None);
    // Refresh no debe tener headers adicionales
    assert!(!refresh_headers.contains_key("Origin"));
    assert!(!refresh_headers.contains_key("X-Requested-With"));
}
