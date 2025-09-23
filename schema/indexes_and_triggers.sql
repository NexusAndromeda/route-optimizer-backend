-- =====================================================
-- ÍNDICES OPTIMIZADOS MULTI-TENANT
-- =====================================================

-- Índices para companies
CREATE INDEX idx_companies_subscription_status ON companies(subscription_status);
CREATE INDEX idx_companies_deleted_at ON companies(deleted_at);

-- Índices para users
CREATE INDEX idx_users_company_id ON users(company_id);
CREATE INDEX idx_users_user_type ON users(user_type);
CREATE INDEX idx_users_user_status ON users(user_status);
CREATE INDEX idx_users_tournee_number ON users(tournee_number);
CREATE INDEX idx_users_deleted_at ON users(deleted_at);
CREATE INDEX idx_users_company_type ON users(company_id, user_type);
CREATE INDEX idx_users_device_token ON users(device_token);
CREATE INDEX idx_users_last_location ON users USING GIST(last_location);
CREATE INDEX idx_users_shift_times ON users(shift_start_time, shift_end_time);

-- Índices para vehicles
CREATE INDEX idx_vehicles_company_id ON vehicles(company_id);
CREATE INDEX idx_vehicles_license_plate ON vehicles(license_plate);
CREATE INDEX idx_vehicles_status ON vehicles(vehicle_status);
CREATE INDEX idx_vehicles_deleted_at ON vehicles(deleted_at);

-- Índices para api_integrations
CREATE INDEX idx_api_integrations_company_id ON api_integrations(company_id);
CREATE INDEX idx_api_integrations_provider_name ON api_integrations(provider_name);
CREATE INDEX idx_api_integrations_sync_status ON api_integrations(sync_status);
CREATE INDEX idx_api_integrations_last_sync ON api_integrations(last_sync_date);

-- Índices para vehicle_documents
CREATE INDEX idx_vehicle_documents_vehicle_id ON vehicle_documents(vehicle_id);
CREATE INDEX idx_vehicle_documents_type ON vehicle_documents(document_type);
CREATE INDEX idx_vehicle_documents_status ON vehicle_documents(document_status);
CREATE INDEX idx_vehicle_documents_expiry_date ON vehicle_documents(expiry_date);
CREATE INDEX idx_vehicle_documents_deleted_at ON vehicle_documents(deleted_at);
CREATE INDEX idx_vehicle_documents_company_status_expiry ON vehicle_documents(company_id, document_status, expiry_date);

-- Índices para vehicle_damages
CREATE INDEX idx_vehicle_damages_vehicle_id ON vehicle_damages(vehicle_id);
CREATE INDEX idx_vehicle_damages_driver_id ON vehicle_damages(driver_id);
CREATE INDEX idx_vehicle_damages_tournee_id ON vehicle_damages(tournee_id);
CREATE INDEX idx_vehicle_damages_type ON vehicle_damages(damage_type);
CREATE INDEX idx_vehicle_damages_status ON vehicle_damages(damage_status);
CREATE INDEX idx_vehicle_damages_incident_date ON vehicle_damages(incident_date);
CREATE INDEX idx_vehicle_damages_deleted_at ON vehicle_damages(deleted_at);

-- Índices para tournees
CREATE INDEX idx_tournees_company_id ON tournees(company_id);
CREATE INDEX idx_tournees_driver_id ON tournees(driver_id);
CREATE INDEX idx_tournees_vehicle_id ON tournees(vehicle_id);
CREATE INDEX idx_tournees_date ON tournees(tournee_date);
CREATE INDEX idx_tournees_status ON tournees(tournee_status);
CREATE INDEX idx_tournees_deleted_at ON tournees(deleted_at);
CREATE INDEX idx_tournees_driver_date ON tournees(driver_id, tournee_date);
CREATE INDEX idx_tournees_company_date ON tournees(company_id, tournee_date);
CREATE INDEX idx_tournees_traffic_conditions ON tournees USING GIN(traffic_conditions);
CREATE INDEX idx_tournees_weather_conditions ON tournees USING GIN(weather_conditions);

-- Índices para packages
CREATE INDEX idx_packages_tournee_id ON packages(tournee_id);
CREATE INDEX idx_packages_tracking_number ON packages(tracking_number);
CREATE INDEX idx_packages_external_tracking ON packages(external_tracking_number);
CREATE INDEX idx_packages_status ON packages(delivery_status);
CREATE INDEX idx_packages_delivery_date ON packages(delivery_date);
CREATE INDEX idx_packages_deleted_at ON packages(deleted_at);
CREATE INDEX idx_packages_delivery_coordinates ON packages USING GIST(delivery_coordinates);
CREATE INDEX idx_packages_company_status_date ON packages(company_id, delivery_status, delivery_date);

-- Índices para driver_field_data
CREATE INDEX idx_driver_field_data_company_id ON driver_field_data(company_id);
CREATE INDEX idx_driver_field_data_driver_id ON driver_field_data(driver_id);
CREATE INDEX idx_driver_field_data_address ON driver_field_data(address);
CREATE INDEX idx_driver_field_data_postal_code ON driver_field_data(postal_code);
CREATE INDEX idx_driver_field_data_city ON driver_field_data(city);
CREATE INDEX idx_driver_field_data_coordinates ON driver_field_data USING GIST(coordinates);
CREATE INDEX idx_driver_field_data_deleted_at ON driver_field_data(deleted_at);

-- Índices para performance_analytics
CREATE INDEX idx_performance_analytics_company_id ON performance_analytics(company_id);
CREATE INDEX idx_performance_analytics_driver_id ON performance_analytics(driver_id);
CREATE INDEX idx_performance_analytics_week_start ON performance_analytics(week_start_date);
CREATE INDEX idx_performance_analytics_week_end ON performance_analytics(week_end_date);
CREATE INDEX idx_performance_analytics_driver_week ON performance_analytics(driver_id, week_start_date);
CREATE INDEX idx_performance_analytics_company_week ON performance_analytics(company_id, week_start_date);

-- Índices para notifications_log
CREATE INDEX idx_notifications_log_company_id ON notifications_log(company_id);
CREATE INDEX idx_notifications_log_admin_id ON notifications_log(admin_id);
CREATE INDEX idx_notifications_log_type ON notifications_log(notification_type);
CREATE INDEX idx_notifications_log_priority ON notifications_log(notification_priority);
CREATE INDEX idx_notifications_log_sent_date ON notifications_log(sent_date);
CREATE INDEX idx_notifications_log_read_status ON notifications_log(read_status);
CREATE INDEX idx_notifications_log_company_type ON notifications_log(company_id, notification_type);
CREATE INDEX idx_notifications_log_unread ON notifications_log(company_id, read_status) WHERE read_status = FALSE;

-- Índices para sync_log
CREATE INDEX idx_sync_log_integration_id ON sync_log(integration_id);
CREATE INDEX idx_sync_log_sync_date ON sync_log(sync_date);
CREATE INDEX idx_sync_log_sync_status ON sync_log(sync_status);
CREATE INDEX idx_sync_log_sync_type ON sync_log(sync_type);
CREATE INDEX idx_sync_log_errors_count ON sync_log(errors_count);
CREATE INDEX idx_sync_log_company_date ON sync_log(company_id, sync_date);

-- Índices para app_versions
CREATE INDEX idx_app_versions_active ON app_versions(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_app_versions_version_code ON app_versions(version_code DESC);
CREATE INDEX idx_app_versions_build_type ON app_versions(build_type);
CREATE INDEX idx_app_versions_created_at ON app_versions(created_at DESC);
CREATE INDEX idx_app_versions_force_update ON app_versions(force_update);
CREATE INDEX idx_app_versions_download_count ON app_versions(download_count DESC);

-- =====================================================
-- FUNCIONES Y TRIGGERS AUTOMÁTICOS
-- =====================================================

-- Función para actualizar updated_at automáticamente
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Función para calcular distancia total de una tournée
CREATE OR REPLACE FUNCTION calculate_tournee_distance()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.end_mileage IS NOT NULL AND NEW.start_mileage IS NOT NULL THEN
        NEW.total_distance = NEW.end_mileage - NEW.start_mileage;
    END IF;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Función para actualizar status de documentos automáticamente
CREATE OR REPLACE FUNCTION update_document_status()
RETURNS TRIGGER AS $$
BEGIN
    -- Actualizar status basado en expiry_date
    IF NEW.expiry_date > CURRENT_DATE + INTERVAL '30 days' THEN
        NEW.document_status = 'valid';
        NEW.notification_sent_30_days = FALSE;
        NEW.notification_sent_15_days = FALSE;
        NEW.notification_sent_expired = FALSE;
    ELSIF NEW.expiry_date > CURRENT_DATE + INTERVAL '15 days' THEN
        NEW.document_status = 'expiring_soon';
        NEW.notification_sent_15_days = FALSE;
        NEW.notification_sent_expired = FALSE;
    ELSIF NEW.expiry_date > CURRENT_DATE THEN
        NEW.document_status = 'expiring_soon';
        NEW.notification_sent_expired = FALSE;
    ELSE
        NEW.document_status = 'expired';
    END IF;
    
    RETURN NEW;
END;
$$ language 'plpgsql';

-- =====================================================
-- TRIGGERS
-- =====================================================

-- Triggers para updated_at en todas las tablas principales
CREATE TRIGGER update_companies_updated_at BEFORE UPDATE ON companies
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_vehicles_updated_at BEFORE UPDATE ON vehicles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_api_integrations_updated_at BEFORE UPDATE ON api_integrations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_vehicle_documents_updated_at BEFORE UPDATE ON vehicle_documents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_vehicle_damages_updated_at BEFORE UPDATE ON vehicle_damages
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_tournees_updated_at BEFORE UPDATE ON tournees
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_packages_updated_at BEFORE UPDATE ON packages
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_driver_field_data_updated_at BEFORE UPDATE ON driver_field_data
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_performance_analytics_updated_at BEFORE UPDATE ON performance_analytics
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_app_versions_updated_at BEFORE UPDATE ON app_versions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Trigger para calcular distancia de tournée
CREATE TRIGGER calculate_tournee_distance_trigger
    BEFORE INSERT OR UPDATE ON tournees
    FOR EACH ROW EXECUTE FUNCTION calculate_tournee_distance();

-- Trigger para actualizar status de documentos
CREATE TRIGGER update_document_status_trigger
    BEFORE INSERT OR UPDATE ON vehicle_documents
    FOR EACH ROW EXECUTE FUNCTION update_document_status();

