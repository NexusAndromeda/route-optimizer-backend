# 🧪 Tests - Backend Testing Suite

Suite completa de tests para el backend de delivery routing.

## 📁 Archivos de Test

### **api_tests.rs** - Tests de API
- Tests de endpoints HTTP
- Validación de respuestas
- Casos de error y éxito
- Solo **web API** (mobile tests eliminados)

### **Cobertura Actual**
- ✅ `POST /api/colis-prive/tournee` - Obtener rutas
- ✅ `POST /api/geocode` - Geocoding
- ✅ Manejo de credenciales inválidas
- ✅ Validación de parámetros

## 🚀 Ejecutar Tests

```bash
# Todos los tests
cargo test

# Tests específicos de API
cargo test api_tests

# Con logs detallados
cargo test -- --nocapture
```

## 📊 Estructura de Test

```rust
#[tokio::test]
async fn test_endpoint_name() {
    // Setup
    let app = create_test_app().await;
    
    // Action
    let response = app.post("/endpoint").json(&request).await;
    
    // Assert
    assert_eq!(response.status(), StatusCode::OK);
}
```

## ✅ Estado

- **Tests web API:** ✅ Funcionando
- **Tests mobile:** ❌ Eliminados (legacy cleanup)
- **Coverage:** Endpoints principales cubiertos