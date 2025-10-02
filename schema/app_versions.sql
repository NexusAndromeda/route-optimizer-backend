-- =====================================================
-- TABLA APP_VERSIONS - GESTIÓN DE VERSIONES DE APK
-- =====================================================
-- Esta tabla es necesaria para el sistema de deploy y actualizaciones
-- de la aplicación móvil

CREATE TABLE app_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    version_name VARCHAR(50) NOT NULL UNIQUE,
    version_code INTEGER NOT NULL UNIQUE CHECK (version_code > 0),
    apk_path VARCHAR(500) NOT NULL,
    download_url VARCHAR(500) NOT NULL,
    release_notes TEXT,
    force_update BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- =====================================================
-- ÍNDICES PARA RENDIMIENTO
-- =====================================================
CREATE INDEX idx_app_versions_active ON app_versions(is_active) WHERE is_active = true;
CREATE INDEX idx_app_versions_version_code ON app_versions(version_code DESC);
CREATE INDEX idx_app_versions_force_update ON app_versions(force_update);

-- =====================================================
-- TRIGGER PARA updated_at
-- =====================================================
CREATE TRIGGER update_app_versions_updated_at
    BEFORE UPDATE ON app_versions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();