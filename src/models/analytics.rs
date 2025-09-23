//! Modelos de Analytics
//! 
//! Este módulo contiene los modelos para métricas de rendimiento,
//! análisis de datos y dashboards.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Métricas de rendimiento del sistema
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PerformanceAnalytics {
    pub id: Uuid,
    pub company_id: Uuid,
    pub tournee_id: Option<Uuid>,
    pub driver_id: Option<Uuid>,
    pub vehicle_id: Option<Uuid>,
    
    // Métricas de tiempo
    pub total_time_minutes: i32,
    pub driving_time_minutes: i32,
    pub waiting_time_minutes: i32,
    
    // Métricas de distancia
    pub total_distance_km: f64,
    pub route_efficiency: f64, // Porcentaje de eficiencia de ruta
    
    // Métricas de entrega
    pub packages_delivered: i32,
    pub packages_failed: i32,
    pub delivery_success_rate: f64,
    
    // Métricas de combustible
    pub fuel_consumed_liters: Option<f64>,
    pub fuel_efficiency_km_l: Option<f64>,
    
    // Métricas de costos
    pub total_cost: Option<Decimal>,
    pub cost_per_package: Option<Decimal>,
    pub cost_per_km: Option<Decimal>,
    
    // Métricas de satisfacción
    pub customer_rating: Option<f64>,
    pub complaints_count: i32,
    
    // Timestamps
    pub date: chrono::NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Resumen para dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub company_id: Uuid,
    pub date: chrono::NaiveDate,
    
    // Resumen de tournées
    pub total_tournees: i32,
    pub completed_tournees: i32,
    pub active_tournees: i32,
    
    // Resumen de paquetes
    pub total_packages: i32,
    pub delivered_packages: i32,
    pub failed_packages: i32,
    
    // Métricas de rendimiento
    pub average_delivery_time_minutes: f64,
    pub average_route_efficiency: f64,
    pub total_distance_km: f64,
    
    // Métricas financieras
    pub total_revenue: Option<Decimal>,
    pub total_costs: Option<Decimal>,
    pub profit_margin: Option<f64>,
    
    // Métricas de satisfacción
    pub average_customer_rating: Option<f64>,
    pub total_complaints: i32,
}

/// Request para crear analytics
#[derive(Debug, Deserialize, Validate)]
pub struct CreateAnalyticsRequest {
    pub tournee_id: Option<Uuid>,
    pub driver_id: Option<Uuid>,
    pub vehicle_id: Option<Uuid>,
    
    #[validate(range(min = 0))]
    pub total_time_minutes: i32,
    
    #[validate(range(min = 0))]
    pub driving_time_minutes: i32,
    
    #[validate(range(min = 0))]
    pub waiting_time_minutes: i32,
    
    #[validate(range(min = 0.0))]
    pub total_distance_km: f64,
    
    #[validate(range(min = 0.0, max = 100.0))]
    pub route_efficiency: f64,
    
    #[validate(range(min = 0))]
    pub packages_delivered: i32,
    
    #[validate(range(min = 0))]
    pub packages_failed: i32,
    
    #[validate(range(min = 0.0, max = 100.0))]
    pub delivery_success_rate: f64,
    
    #[validate(range(min = 0.0))]
    pub fuel_consumed_liters: Option<f64>,
    
    #[validate(range(min = 0.0))]
    pub fuel_efficiency_km_l: Option<f64>,
    
    #[validate(range(min = 0.0))]
    pub total_cost: Option<f64>,
    
    #[validate(range(min = 0.0))]
    pub cost_per_package: Option<f64>,
    
    #[validate(range(min = 0.0))]
    pub cost_per_km: Option<f64>,
    
    #[validate(range(min = 1.0, max = 5.0))]
    pub customer_rating: Option<f64>,
    
    #[validate(range(min = 0))]
    pub complaints_count: i32,
}

/// Request para actualizar analytics
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAnalyticsRequest {
    pub tournee_id: Option<Uuid>,
    pub driver_id: Option<Uuid>,
    pub vehicle_id: Option<Uuid>,
    
    #[validate(range(min = 0))]
    pub total_time_minutes: Option<i32>,
    
    #[validate(range(min = 0))]
    pub driving_time_minutes: Option<i32>,
    
    #[validate(range(min = 0))]
    pub waiting_time_minutes: Option<i32>,
    
    #[validate(range(min = 0.0))]
    pub total_distance_km: Option<f64>,
    
    #[validate(range(min = 0.0, max = 100.0))]
    pub route_efficiency: Option<f64>,
    
    #[validate(range(min = 0))]
    pub packages_delivered: Option<i32>,
    
    #[validate(range(min = 0))]
    pub packages_failed: Option<i32>,
    
    #[validate(range(min = 0.0, max = 100.0))]
    pub delivery_success_rate: Option<f64>,
    
    #[validate(range(min = 0.0))]
    pub fuel_consumed_liters: Option<f64>,
    
    #[validate(range(min = 0.0))]
    pub fuel_efficiency_km_l: Option<f64>,
    
    #[validate(range(min = 0.0))]
    pub total_cost: Option<f64>,
    
    #[validate(range(min = 0.0))]
    pub cost_per_package: Option<f64>,
    
    #[validate(range(min = 0.0))]
    pub cost_per_km: Option<f64>,
    
    #[validate(range(min = 1.0, max = 5.0))]
    pub customer_rating: Option<f64>,
    
    #[validate(range(min = 0))]
    pub complaints_count: Option<i32>,
}

/// Response de analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsResponse {
    pub id: Uuid,
    pub company_id: Uuid,
    pub tournee_id: Option<Uuid>,
    pub driver_id: Option<Uuid>,
    pub vehicle_id: Option<Uuid>,
    
    // Métricas de tiempo
    pub total_time_minutes: i32,
    pub driving_time_minutes: i32,
    pub waiting_time_minutes: i32,
    
    // Métricas de distancia
    pub total_distance_km: f64,
    pub route_efficiency: f64,
    
    // Métricas de entrega
    pub packages_delivered: i32,
    pub packages_failed: i32,
    pub delivery_success_rate: f64,
    
    // Métricas de combustible
    pub fuel_consumed_liters: Option<f64>,
    pub fuel_efficiency_km_l: Option<f64>,
    
    // Métricas de costos
    pub total_cost: Option<Decimal>,
    pub cost_per_package: Option<Decimal>,
    pub cost_per_km: Option<Decimal>,
    
    // Métricas de satisfacción
    pub customer_rating: Option<f64>,
    pub complaints_count: i32,
    
    // Timestamps
    pub date: chrono::NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Filtros para analytics
#[derive(Debug, Clone, Deserialize)]
pub struct AnalyticsFilters {
    pub company_id: Uuid,
    pub tournee_id: Option<Uuid>,
    pub driver_id: Option<Uuid>,
    pub vehicle_id: Option<Uuid>,
    pub date_from: Option<chrono::NaiveDate>,
    pub date_to: Option<chrono::NaiveDate>,
    pub min_efficiency: Option<f64>,
    pub max_efficiency: Option<f64>,
    pub min_rating: Option<f64>,
    pub max_rating: Option<f64>,
}

/// Lista paginada de analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsListResponse {
    pub analytics: Vec<AnalyticsResponse>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}
