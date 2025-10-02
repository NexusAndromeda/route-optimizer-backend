-- =====================================================
-- SCHEMA FINAL SIMPLIFICADO - DELIVERY ROUTING
-- =====================================================
-- Enfocado en gestión de empresa, vehículos y direcciones
-- con referencias de tournées para organizar datos

-- EXTENSIONES
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "postgis";

-- =====================================================
-- 1. COMPANIES
-- =====================================================
CREATE TABLE companies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    address TEXT NOT NULL,
    siret VARCHAR(14) UNIQUE,
    subscription_plan VARCHAR(50) DEFAULT 'basic',
    subscription_status VARCHAR(20) DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- =====================================================
-- 2. USERS (solo jefes)
-- =====================================================
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- =====================================================
-- 3. VEHICLES
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
-- 4. VEHICLE_DOCUMENTS
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
-- 5. ROUTES (solo para referencias de tournées)
-- =====================================================
CREATE TABLE routes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    vehicle_id UUID REFERENCES vehicles(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- =====================================================
-- 6. VEHICLE_DAMAGES
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
-- 7. ADDRESSES (datos de campo)
-- =====================================================
CREATE TABLE addresses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    route_id UUID NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
    address TEXT NOT NULL,
    postal_code VARCHAR(20),
    coordinates GEOMETRY(Point, 4326),
    door_codes TEXT,
    mailbox_access BOOLEAN DEFAULT FALSE,
    access_instructions TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);