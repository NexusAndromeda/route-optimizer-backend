-- =====================================================
-- SCHEMA COMPLETO CORREGIDO - DELIVERY ROUTING
-- =====================================================
-- Convenciones: Primary keys = 'id', Foreign keys = 'tabla_id'
-- Arquitectura multi-tenant jerárquica

-- EXTENSIONES
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "postgis";

-- =====================================================
-- NIVEL 1 - COMPANIES (Tabla raíz)
-- =====================================================
CREATE TABLE companies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    address TEXT NOT NULL,
    subscription_plan VARCHAR(50) NOT NULL DEFAULT 'basic',
    subscription_status VARCHAR(20) NOT NULL DEFAULT 'active',
    max_drivers INTEGER NOT NULL DEFAULT 10,
    max_vehicles INTEGER NOT NULL DEFAULT 5,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE
);

-- =====================================================
-- NIVEL 2A - USERS
-- =====================================================
CREATE TYPE user_type AS ENUM ('admin', 'driver');
CREATE TYPE user_status AS ENUM ('active', 'inactive', 'suspended');

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    user_type user_type NOT NULL,
    user_status user_status NOT NULL DEFAULT 'active',
    
    -- Campos comunes
    username VARCHAR(100) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(20),
    
    -- Específicos para drivers
    tournee_number VARCHAR(20),
    driver_license VARCHAR(50),
    hire_date DATE,
    device_token VARCHAR(255),
    last_location POINT,
    shift_start_time TIME,
    shift_end_time TIME,
    
    -- Específicos para admins
    permissions JSONB DEFAULT '{}',
    
    -- Metadatos
    last_login TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    
    -- Constraints
    CONSTRAINT unique_username_per_company UNIQUE (company_id, username),
    CONSTRAINT unique_email_per_company UNIQUE (company_id, email)
);

-- =====================================================
-- NIVEL 2B - VEHICLES
-- =====================================================
CREATE TYPE vehicle_status AS ENUM ('active', 'maintenance', 'out_of_service', 'retired');

CREATE TABLE vehicles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    license_plate VARCHAR(20) NOT NULL,
    brand VARCHAR(100) NOT NULL,
    model VARCHAR(100) NOT NULL,
    year INTEGER,
    color VARCHAR(50),
    
    -- Estado operativo
    vehicle_status vehicle_status NOT NULL DEFAULT 'active',
    current_mileage DECIMAL(10,2) NOT NULL DEFAULT 0,
    fuel_type VARCHAR(20) NOT NULL DEFAULT 'diesel',
    fuel_capacity DECIMAL(5,2),
    weekly_fuel_allocation DECIMAL(5,2),
    
    -- Métricas de daños
    total_damage_cost DECIMAL(10,2) NOT NULL DEFAULT 0,
    damage_incidents_count INTEGER NOT NULL DEFAULT 0,
    
    -- Información técnica
    vin VARCHAR(17),
    engine_size VARCHAR(20),
    transmission VARCHAR(20),
    
    -- Metadatos
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    
    -- Constraints
    CONSTRAINT unique_license_plate_per_company UNIQUE (company_id, license_plate),
    CONSTRAINT positive_mileage CHECK (current_mileage >= 0)
);

-- =====================================================
-- NIVEL 2C - API_INTEGRATIONS
-- =====================================================
CREATE TYPE sync_status AS ENUM ('active', 'error', 'disabled', 'syncing');

CREATE TABLE api_integrations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    
    -- Información del proveedor
    provider_name VARCHAR(100) NOT NULL,
    provider_display_name VARCHAR(255),
    api_version VARCHAR(20),
    
    -- Credenciales y configuración
    api_credentials JSONB NOT NULL,
    api_endpoint TEXT,
    webhook_url TEXT,
    
    -- Estado de sincronización
    sync_status sync_status NOT NULL DEFAULT 'active',
    last_sync_date TIMESTAMP WITH TIME ZONE,
    last_successful_sync TIMESTAMP WITH TIME ZONE,
    consecutive_errors INTEGER DEFAULT 0,
    
    -- Límites y frecuencia
    daily_sync_limit INTEGER DEFAULT 1000,
    sync_frequency_hours INTEGER DEFAULT 24,
    max_retry_attempts INTEGER DEFAULT 3,
    
    -- Configuración específica del proveedor
    provider_config JSONB DEFAULT '{}',
    field_mappings JSONB DEFAULT '{}',
    
    -- Metadatos
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    
    -- Constraints
    CONSTRAINT unique_provider_per_company UNIQUE (company_id, provider_name)
);

-- =====================================================
-- NIVEL 3A - VEHICLE_DOCUMENTS
-- =====================================================
CREATE TYPE document_type AS ENUM (
    'technical_control', 'insurance', 'carte_grise', 
    'driver_license', 'vehicle_registration', 'maintenance_book'
);

CREATE TYPE document_status AS ENUM (
    'valid', 'expiring_soon', 'expired', 
    'renewal_in_progress', 'missing'
);

CREATE TABLE vehicle_documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    vehicle_id UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    document_type document_type NOT NULL,
    document_number VARCHAR(100),
    document_name VARCHAR(255) NOT NULL,
    
    -- Fechas importantes
    issue_date DATE,
    expiry_date DATE NOT NULL,
    renewal_reminder_date DATE,
    
    -- Estado y notificaciones
    document_status document_status NOT NULL DEFAULT 'valid',
    notification_sent_30_days BOOLEAN DEFAULT FALSE,
    notification_sent_15_days BOOLEAN DEFAULT FALSE,
    notification_sent_expired BOOLEAN DEFAULT FALSE,
    
    -- Archivos y metadatos
    document_path TEXT,
    document_url TEXT,
    file_size BIGINT,
    mime_type VARCHAR(100),
    
    -- Notas y estado
    notes TEXT,
    renewal_notes TEXT,
    insurance_company VARCHAR(100),
    policy_number VARCHAR(100),
    
    -- Metadatos
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    
    -- Constraints
    CONSTRAINT unique_document_type_per_vehicle UNIQUE (vehicle_id, document_type)
);

-- =====================================================
-- NIVEL 3B - VEHICLE_DAMAGES
-- =====================================================
CREATE TYPE damage_type AS ENUM (
    'scratch', 'dent', 'mechanical', 'accident', 
    'traffic_fine', 'vandalism', 'weather_damage', 'wear_and_tear'
);

CREATE TYPE damage_status AS ENUM (
    'pending', 'assessed', 'repaired', 'driver_liable',
    'insurance_claim', 'closed'
);

CREATE TABLE vehicle_damages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    vehicle_id UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    driver_id UUID REFERENCES users(id) ON DELETE SET NULL,
    tournee_id UUID,
    
    -- Información del incidente
    incident_date DATE NOT NULL,
    incident_time TIME,
    damage_type damage_type NOT NULL,
    damage_location VARCHAR(255),
    description TEXT NOT NULL,
    
    -- Costos y responsabilidad
    estimated_repair_cost DECIMAL(10,2),
    actual_repair_cost DECIMAL(10,2),
    responsibility_percentage INTEGER CHECK (responsibility_percentage >= 0 AND responsibility_percentage <= 100),
    driver_deductible DECIMAL(10,2),
    
    -- Estado y seguimiento
    damage_status damage_status NOT NULL DEFAULT 'pending',
    assessment_date DATE,
    repair_date DATE,
    completion_date DATE,
    
    -- Evidencia y documentación
    photo_evidence TEXT[],
    police_report TEXT,
    witness_statements TEXT,
    
    -- Seguro
    insurance_claim BOOLEAN DEFAULT FALSE,
    claim_number VARCHAR(100),
    deductible_amount DECIMAL(10,2),
    insurance_company VARCHAR(100),
    
    -- Metadatos
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE
);

-- =====================================================
-- NIVEL 3C - TOURNEES
-- =====================================================
CREATE TYPE tournee_status AS ENUM (
    'pending', 'in_progress', 'completed', 'cancelled', 'paused'
);

CREATE TABLE tournees (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    driver_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    vehicle_id UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    
    -- Información de la ruta
    tournee_date DATE NOT NULL,
    tournee_number VARCHAR(50),
    start_location TEXT,
    end_location TEXT,
    
    -- Estado operativo
    tournee_status tournee_status NOT NULL DEFAULT 'pending',
    start_time TIMESTAMP WITH TIME ZONE,
    end_time TIMESTAMP WITH TIME ZONE,
    
    -- Métricas de kilometraje y combustible
    start_mileage DECIMAL(10,2),
    end_mileage DECIMAL(10,2),
    total_distance DECIMAL(8,2),
    fuel_consumed DECIMAL(5,2),
    fuel_cost DECIMAL(8,2),
    
    -- Inspecciones
    pre_inspection_notes TEXT,
    post_inspection_notes TEXT,
    pre_inspection_photos TEXT[],
    post_inspection_photos TEXT[],
    
    -- Optimización de ruta
    route_optimization_score DECIMAL(3,2),
    estimated_duration_minutes INTEGER,
    actual_duration_minutes INTEGER,
    
    -- Ruta y condiciones
    route_coordinates TEXT[],
    traffic_conditions JSONB,
    weather_conditions JSONB,
    
    -- Origen de la tournée
    tournee_origin VARCHAR(50) DEFAULT 'manual',
    external_tournee_id VARCHAR(100),
    integration_id UUID REFERENCES api_integrations(id) ON DELETE SET NULL,
    
    -- Metadatos
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    
    -- Constraints
    CONSTRAINT unique_tournee_per_driver_date UNIQUE (driver_id, tournee_date)
);

-- =====================================================
-- NIVEL 4A - PACKAGES
-- =====================================================
CREATE TYPE delivery_status AS ENUM (
    'pending', 'in_transit', 'out_for_delivery', 
    'delivered', 'failed', 'returned', 'cancelled'
);

CREATE TYPE delivery_failure_reason AS ENUM (
    'recipient_not_home', 'wrong_address', 'package_damaged',
    'refused_delivery', 'security_restriction', 'weather_conditions',
    'vehicle_breakdown', 'driver_emergency'
);

CREATE TABLE packages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    tournee_id UUID NOT NULL REFERENCES tournees(id) ON DELETE CASCADE,
    
    -- Información del paquete
    tracking_number VARCHAR(100) NOT NULL,
    external_tracking_number VARCHAR(100),
    package_origin VARCHAR(50) DEFAULT 'manual',
    external_package_id VARCHAR(100),
    integration_id UUID REFERENCES api_integrations(id) ON DELETE SET NULL,
    package_type VARCHAR(100),
    package_weight DECIMAL(6,2),
    package_dimensions VARCHAR(50),
    
    -- Estado de entrega
    delivery_status delivery_status NOT NULL DEFAULT 'pending',
    delivery_date DATE,
    delivery_time TIME,
    delivery_attempts INTEGER DEFAULT 0,
    
    -- Información de entrega
    recipient_name VARCHAR(255),
    recipient_phone VARCHAR(20),
    delivery_address TEXT NOT NULL,
    delivery_instructions TEXT,
    
    -- Fallos y reintentos
    failure_reason delivery_failure_reason,
    failure_notes TEXT,
    reschedule_date DATE,
    
    -- Evidencia de entrega
    delivery_photo TEXT,
    signature_required BOOLEAN DEFAULT FALSE,
    signature_image TEXT,
    signature_photo TEXT,
    
    -- Ubicación y tiempo de entrega
    delivery_coordinates POINT,
    delivery_duration_minutes INTEGER,
    
    -- Notas del chofer
    driver_notes TEXT,
    package_condition TEXT,
    
    -- Metadatos
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    
    -- Constraints
    CONSTRAINT unique_tracking_per_tournee UNIQUE (tournee_id, tracking_number)
);

-- =====================================================
-- NIVEL 4B - DRIVER_FIELD_DATA
-- =====================================================
CREATE TABLE driver_field_data (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    driver_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Ubicación
    address TEXT NOT NULL,
    postal_code VARCHAR(20),
    city VARCHAR(100),
    coordinates POINT,
    
    -- Información de acceso
    door_codes TEXT,
    access_instructions TEXT,
    security_notes TEXT,
    
    -- Estado del buzón
    mailbox_location TEXT,
    mailbox_working BOOLEAN,
    mailbox_issues TEXT,
    
    -- Horarios y preferencias
    preferred_delivery_time VARCHAR(100),
    delivery_restrictions TEXT,
    special_instructions TEXT,
    
    -- Calidad de los datos
    confidence_score INTEGER CHECK (confidence_score >= 1 AND confidence_score <= 5),
    data_source VARCHAR(50) DEFAULT 'driver_input',
    verification_count INTEGER DEFAULT 1,
    
    -- Última actualización
    last_updated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    last_verified_date DATE,
    
    -- Metadatos
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    
    -- Constraints
    CONSTRAINT unique_address_per_company UNIQUE (company_id, address)
);

-- =====================================================
-- NIVEL 5A - PERFORMANCE_ANALYTICS
-- =====================================================
CREATE TABLE performance_analytics (
    id UUID NOT NULL DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    driver_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Período de análisis
    week_start_date DATE NOT NULL,
    week_end_date DATE NOT NULL,
    
    -- Métricas de entrega
    total_packages INTEGER NOT NULL DEFAULT 0,
    successful_deliveries INTEGER NOT NULL DEFAULT 0,
    failed_deliveries INTEGER NOT NULL DEFAULT 0,
    delivery_success_rate DECIMAL(5,2),
    
    -- Métricas de distancia y combustible
    km_driven DECIMAL(8,2) NOT NULL DEFAULT 0,
    fuel_consumed DECIMAL(5,2) NOT NULL DEFAULT 0,
    fuel_cost DECIMAL(8,2) NOT NULL DEFAULT 0,
    fuel_efficiency DECIMAL(6,2),
    
    -- Métricas de tiempo
    total_working_hours DECIMAL(4,2),
    average_delivery_time_minutes DECIMAL(5,2),
    route_optimization_score DECIMAL(3,2),
    
    -- Métricas de daños y incidentes
    damage_incidents INTEGER NOT NULL DEFAULT 0,
    total_damage_cost DECIMAL(10,2) NOT NULL DEFAULT 0,
    damage_score DECIMAL(3,2),
    
    -- Métricas de rendimiento
    efficiency_ratio DECIMAL(5,2),
    cost_per_package DECIMAL(6,2),
    profit_margin DECIMAL(5,2),
    
    -- Banderas de anomalías
    anomaly_flags JSONB DEFAULT '{}',
    
    -- Metadatos
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT unique_driver_week UNIQUE (driver_id, week_start_date)
);

-- =====================================================
-- NIVEL 5B - NOTIFICATIONS_LOG
-- =====================================================
CREATE TYPE notification_type AS ENUM (
    'expiry_warning_30', 'expiry_warning_15', 'expired_critical',
    'damage_incident', 'performance_alert', 'fuel_consumption_alert',
    'maintenance_reminder', 'system_alert'
);

CREATE TYPE notification_priority AS ENUM ('low', 'medium', 'high', 'critical');

CREATE TABLE notifications_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    admin_id UUID REFERENCES users(id) ON DELETE SET NULL,
    document_id UUID REFERENCES vehicle_documents(id) ON DELETE SET NULL,
    vehicle_id UUID REFERENCES vehicles(id) ON DELETE SET NULL,
    driver_id UUID REFERENCES users(id) ON DELETE SET NULL,
    
    -- Información de la notificación
    notification_type notification_type NOT NULL,
    notification_priority notification_priority NOT NULL DEFAULT 'medium',
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    
    -- Estado y seguimiento
    sent_date TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    read_status BOOLEAN DEFAULT FALSE,
    read_date TIMESTAMP WITH TIME ZONE,
    action_taken TEXT,
    action_date TIMESTAMP WITH TIME ZONE,
    
    -- Metadatos adicionales
    metadata JSONB DEFAULT '{}',
    email_sent BOOLEAN DEFAULT FALSE,
    sms_sent BOOLEAN DEFAULT FALSE,
    push_notification_sent BOOLEAN DEFAULT FALSE,
    
    -- Metadatos
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- =====================================================
-- NIVEL 5C - SYNC_LOG
-- =====================================================
CREATE TABLE sync_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    integration_id UUID NOT NULL REFERENCES api_integrations(id) ON DELETE CASCADE,
    
    -- Información de la sincronización
    sync_date TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    sync_type VARCHAR(50) NOT NULL,
    sync_direction VARCHAR(20) NOT NULL,
    
    -- Métricas de la sincronización
    records_processed INTEGER NOT NULL DEFAULT 0,
    records_created INTEGER DEFAULT 0,
    records_updated INTEGER DEFAULT 0,
    records_deleted INTEGER DEFAULT 0,
    records_failed INTEGER DEFAULT 0,
    errors_count INTEGER NOT NULL DEFAULT 0,
    
    -- Performance y duración
    sync_duration_seconds INTEGER,
    sync_start_time TIMESTAMP WITH TIME ZONE,
    sync_end_time TIMESTAMP WITH TIME ZONE,
    
    -- Detalles de errores y estado
    error_details JSONB DEFAULT '{}',
    sync_status VARCHAR(20) NOT NULL DEFAULT 'completed',
    retry_count INTEGER DEFAULT 0,
    
    -- Metadatos adicionales
    api_response_code INTEGER,
    api_response_time_ms INTEGER,
    data_size_bytes BIGINT,
    
    -- Metadatos
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- =====================================================
-- NIVEL 6 - APP VERSIONS (Sistema de actualizaciones)
-- =====================================================
CREATE TABLE app_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Información de la versión
    version_name VARCHAR(50) NOT NULL,
    version_code INTEGER NOT NULL,
    build_type VARCHAR(20) NOT NULL DEFAULT 'release',
    
    -- Archivos y descarga
    apk_path VARCHAR(500) NOT NULL,
    apk_size_bytes BIGINT,
    apk_checksum VARCHAR(64), -- SHA-256 hash para verificación
    
    -- URLs de descarga
    download_url VARCHAR(500) NOT NULL,
    backup_download_url VARCHAR(500),
    
    -- Información de la release
    release_notes TEXT,
    release_notes_fr TEXT, -- Versión en francés
    release_notes_en TEXT, -- Versión en inglés
    
    -- Configuración de actualización
    force_update BOOLEAN NOT NULL DEFAULT FALSE,
    min_supported_version VARCHAR(50),
    max_supported_version VARCHAR(50),
    
    -- Compatibilidad
    min_android_version INTEGER, -- API level mínimo
    max_android_version INTEGER, -- API level máximo
    supported_architectures TEXT[], -- ['arm64-v8a', 'armeabi-v7a']
    
    -- Estado y disponibilidad
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_beta BOOLEAN NOT NULL DEFAULT FALSE,
    is_rollback BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Estadísticas de descarga
    download_count INTEGER NOT NULL DEFAULT 0,
    last_downloaded_at TIMESTAMP WITH TIME ZONE,
    
    -- Metadatos
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    published_at TIMESTAMP WITH TIME ZONE,
    deprecated_at TIMESTAMP WITH TIME ZONE,
    
    -- Constraints
    CONSTRAINT unique_version_code UNIQUE (version_code),
    CONSTRAINT unique_version_name UNIQUE (version_name),
    CONSTRAINT positive_version_code CHECK (version_code > 0),
    CONSTRAINT positive_apk_size CHECK (apk_size_bytes > 0)
);

-- Índices para app_versions
CREATE INDEX idx_app_versions_active ON app_versions(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_app_versions_version_code ON app_versions(version_code DESC);
CREATE INDEX idx_app_versions_build_type ON app_versions(build_type);
CREATE INDEX idx_app_versions_created_at ON app_versions(created_at DESC);

