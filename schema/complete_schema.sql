-- =====================================================
-- SCHEMA FINAL SIMPLIFICADO - DELIVERY ROUTING
-- =====================================================
-- Enfocado en gestión de empresa (con admin integrado),
-- vehículos y direcciones con referencias de tournées
-- para organizar datos

-- EXTENSIONES
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "postgis";

-- =====================================================
-- 1. COMPANIES (incluye admin/jefe de empresa)
-- =====================================================
CREATE TABLE companies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    address TEXT NOT NULL,
    siret VARCHAR(14) UNIQUE,
    
    -- Admin/Jefe de empresa
    admin_full_name VARCHAR(255) NOT NULL,
    admin_email VARCHAR(255) UNIQUE NOT NULL,
    admin_password_hash VARCHAR(255) NOT NULL,
    
    -- Subscription info
    subscription_plan VARCHAR(50) DEFAULT 'basic',
    subscription_status VARCHAR(20) DEFAULT 'active',
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- =====================================================
-- 2. VEHICLES
-- =====================================================
CREATE TABLE vehicles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    license_plate VARCHAR(20) NOT NULL,
    brand VARCHAR(100),
    model VARCHAR(100),
    vehicle_status VARCHAR(20) DEFAULT 'active',
    current_mileage DECIMAL(10,2) DEFAULT 0,
    fuel_type VARCHAR(20) DEFAULT 'diesel',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- =====================================================
-- 3. VEHICLE_DOCUMENTS
-- =====================================================
CREATE TABLE vehicle_documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    vehicle_id UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    document_type VARCHAR(50) NOT NULL, -- 'technical_control', 'insurance', 'carte_grise'
    document_number VARCHAR(100),
    expiry_date DATE NOT NULL,
    document_status VARCHAR(20) DEFAULT 'valid',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- =====================================================
-- 4. ROUTES (solo para referencias de tournées)
-- =====================================================
CREATE TABLE routes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    vehicle_id UUID REFERENCES vehicles(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- =====================================================
-- 5. VEHICLE_DAMAGES
-- =====================================================
CREATE TABLE vehicle_damages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    vehicle_id UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    route_id UUID REFERENCES routes(id) ON DELETE SET NULL,
    damage_type VARCHAR(100) NOT NULL, -- 'accident', 'wear', 'maintenance'
    damage_status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'repaired', 'written_off'
    repair_cost DECIMAL(10,2),
    incident_date DATE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- =====================================================
-- 6. ADDRESSES (datos de campo - NUEVA ESTRUCTURA)
-- =====================================================
CREATE TABLE addresses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID REFERENCES companies(id) ON DELETE CASCADE,
    
    -- Datos oficiales de la API francesa
    official_label TEXT NOT NULL UNIQUE,        -- "4 Rue Gaston Tissandier 75018 Paris"
    street_name VARCHAR(255) NOT NULL,          -- "Rue Gaston Tissandier"
    street_number VARCHAR(20),                  -- "4"
    postcode VARCHAR(20) NOT NULL,              -- "75018"
    city VARCHAR(100) NOT NULL,                 -- "Paris"
    coordinates GEOMETRY(Point, 4326) NOT NULL,
    
    -- Datos del chofer (compartidos para esta dirección)
    door_code TEXT,                             -- Código de puerta del edificio
    has_mailbox_access BOOLEAN DEFAULT FALSE,   -- Acceso a buzón
    driver_notes TEXT,                          -- "Apto 12: buzón bloqueado"
    
    -- Metadata
    last_updated_by VARCHAR(100),               -- Matricule del chofer que actualizó
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Índices para búsqueda rápida
    INDEX idx_street_postcode (street_name, postcode),
    INDEX idx_coordinates (coordinates),
    INDEX idx_postcode (postcode)
);