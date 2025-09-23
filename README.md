# 🦀 Route Optimizer Backend

Backend en Rust con Axum para el sistema de optimización de rutas de entrega para Colis Privé.

## 📁 Estructura del Proyecto

```
backend/
├── src/                    # Código fuente principal
│   ├── api/               # Endpoints y controladores
│   │   ├── colis_prive/   # Integración con Colis Privé
│   │   ├── auth/          # Autenticación
│   │   ├── packages/      # Gestión de paquetes
│   │   └── companies/     # Gestión de empresas
│   ├── models/            # Estructuras de datos
│   ├── services/          # Lógica de negocio
│   ├── cache/             # Sistema de cache Redis
│   ├── middleware/        # Middleware personalizado
│   └── utils/             # Utilidades
├── schema/                # Esquemas de base de datos
├── tests/                 # Tests unitarios e integración
├── tools/                 # Scripts de análisis
└── Cargo.toml            # Configuración Rust
```

## 🚀 Características

- **API REST** con Axum framework
- **Integración Colis Privé** API completa
- **Optimización híbrida** de rutas
- **Base de datos** PostgreSQL con PostGIS
- **Cache Redis** para optimización
- **Autenticación JWT** segura
- **Geocoding** con Mapbox
- **Sistema de empresas** multi-tenant

## 🔧 Instalación y Uso

```bash
# Instalar dependencias
cargo build

# Ejecutar en desarrollo
cargo run

# Ejecutar tests
cargo test

# Compilar para producción
cargo build --release
```

## 🌐 Endpoints Principales

### Autenticación
- `POST /api/colis-prive/auth` - Autenticación de usuarios
- `GET /api/colis-prive/companies` - Lista de empresas

### Paquetes y Rutas
- `GET /api/colis-prive/packages` - Obtener paquetes del día
- `POST /api/colis-prive/optimize` - Optimizar ruta con Colis Privé
- `POST /api/colis-prive/reorder` - Reordenar paquetes manualmente

### Sistema
- `GET /api/health` - Estado del sistema
- `GET /api/analytics` - Métricas y estadísticas

## 🔒 Configuración

Variables de entorno requeridas en `.env`:

```bash
# Base de datos
DATABASE_URL=postgresql://user:password@localhost:5432/delivery_routing

# Redis
REDIS_URL=redis://localhost:6379

# Colis Privé
COLIS_PRIVE_BASE_URL=https://wstournee-v2.colisprive.com
COLIS_PRIVE_SSO_URL=https://sso.colisprive.com

# Mapbox
MAPBOX_ACCESS_TOKEN=your_mapbox_token

# JWT
JWT_SECRET=your_jwt_secret_key

# Servidor
HOST=0.0.0.0
PORT=8000
```

## 🗄️ Base de Datos

El proyecto incluye esquemas SQL completos:

- `schema/complete_schema.sql` - Esquema principal
- `schema/indexes_and_triggers.sql` - Índices y triggers
- `schema/sample_app_versions.sql` - Datos de ejemplo

## 🧪 Testing

```bash
# Tests unitarios
cargo test

# Tests de integración
cargo test --test api_tests

# Tests específicos de Colis Privé
cargo test --test colis_prive_integration
```

## 📊 Monitoreo

- **Logs estructurados** con diferentes niveles
- **Métricas de performance** integradas
- **Health checks** automáticos
- **Cache hit/miss** statistics

## 🚀 Deployment

```bash
# Compilar para producción
cargo build --release

# Ejecutar con variables de entorno
DATABASE_URL=... REDIS_URL=... ./target/release/delivery_routing
```

## 🔧 Herramientas de Desarrollo

- `tools/analyze_json.rs` - Análisis de datos JSON
- `tools/classify_deliveries.rs` - Clasificación de entregas
- `scripts/manage_versions.rs` - Gestión de versiones