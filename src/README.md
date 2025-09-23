# 📦 Source Code - Backend

Código fuente principal del backend en Rust.

## 📁 Organización

### 🌐 **api/** - Endpoints y Controladores
- `colis_prive.rs` - API endpoints para Colis Privé
- `mod.rs` - Módulo principal de API

### 📊 **models/** - Estructuras de Datos  
- `colis_prive_v3_models.rs` - Modelos para API v3
- `delivery_models.rs` - Modelos de entrega
- `mod.rs` - Exportación de modelos

### ⚙️ **services/** - Lógica de Negocio
- `colis_prive_flow_service.rs` - Flujo principal Colis Privé
- `colis_prive_complete_flow_service.rs` - Flujo completo
- `geocoding_service.rs` - Servicios de geocoding
- `mod.rs` - Servicios disponibles

### 🚀 **main.rs** - Punto de Entrada
- Configuración del servidor Axum
- Inicialización de rutas
- Configuración de middleware

## 🔄 Flujo de Datos

```
HTTP Request → API Layer → Service Layer → Models → Response
```

## 🛡️ Principios

- **Separación de responsabilidades**
- **Código limpio** y documentado  
- **Manejo de errores** robusto
- **Solo API web** (mobile legacy eliminado)