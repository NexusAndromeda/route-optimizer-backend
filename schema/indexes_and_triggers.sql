-- =====================================================
-- ÍNDICES Y TRIGGERS - SCHEMA SIMPLIFICADO
-- =====================================================

-- =====================================================
-- ÍNDICES PARA RENDIMIENTO
-- =====================================================

-- Índices para relaciones principales
CREATE INDEX idx_users_company_id ON users(company_id);
CREATE INDEX idx_vehicles_company_id ON vehicles(company_id);
CREATE INDEX idx_vehicle_documents_vehicle_id ON vehicle_documents(vehicle_id);
CREATE INDEX idx_routes_company_id ON routes(company_id);
CREATE INDEX idx_routes_vehicle_id ON routes(vehicle_id);
CREATE INDEX idx_vehicle_damages_vehicle_id ON vehicle_damages(vehicle_id);
CREATE INDEX idx_vehicle_damages_route_id ON vehicle_damages(route_id);
CREATE INDEX idx_addresses_route_id ON addresses(route_id);

-- Índice espacial para coordenadas (PostGIS)
CREATE INDEX idx_addresses_coordinates ON addresses USING GIST(coordinates);

-- Índices para búsquedas frecuentes
CREATE INDEX idx_companies_siret ON companies(siret);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_vehicles_license_plate ON vehicles(license_plate);
CREATE INDEX idx_vehicle_documents_expiry_date ON vehicle_documents(expiry_date);
CREATE INDEX idx_vehicle_damages_incident_date ON vehicle_damages(incident_date);

-- =====================================================
-- TRIGGERS PARA updated_at
-- =====================================================

-- Función para actualizar updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers para cada tabla (solo las que tengan updated_at)
-- Nota: En este schema simplificado no tenemos updated_at,
-- pero mantenemos la función por si la necesitamos en el futuro

-- =====================================================
-- VISTAS ÚTILES
-- =====================================================

-- Vista para vehículos con información de empresa
CREATE VIEW vehicles_with_company AS
SELECT 
    v.id,
    v.license_plate,
    v.brand,
    v.model,
    v.vehicle_status,
    v.current_mileage,
    v.fuel_type,
    c.name as company_name,
    c.siret as company_siret,
    v.created_at
FROM vehicles v
JOIN companies c ON v.company_id = c.id;

-- Vista para direcciones con información de ruta
CREATE VIEW addresses_with_route AS
SELECT 
    a.id,
    a.address,
    a.postal_code,
    a.coordinates,
    a.door_codes,
    a.mailbox_access,
    a.access_instructions,
    r.id as route_id,
    v.license_plate,
    c.name as company_name,
    a.created_at
FROM addresses a
JOIN routes r ON a.route_id = r.id
JOIN vehicles v ON r.vehicle_id = v.id
JOIN companies c ON r.company_id = c.id;

-- Vista para daños de vehículos con información completa
CREATE VIEW vehicle_damages_with_details AS
SELECT 
    vd.id,
    vd.damage_type,
    vd.damage_status,
    vd.repair_cost,
    vd.incident_date,
    v.license_plate,
    v.brand,
    v.model,
    c.name as company_name,
    r.id as route_id,
    vd.created_at
FROM vehicle_damages vd
JOIN vehicles v ON vd.vehicle_id = v.id
JOIN companies c ON v.company_id = c.id
LEFT JOIN routes r ON vd.route_id = r.id;