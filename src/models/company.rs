//! Modelo de Company
//! 
//! Este módulo contiene el struct Company y sus variantes para CRUD operations.
//! Mapea exactamente al schema PostgreSQL con primary key 'id'.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Company principal - mapea exactamente a la tabla companies
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Company {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub subscription_plan: String,
    pub subscription_status: String,
    pub max_drivers: i32,
    pub max_vehicles: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Request para crear una nueva company
#[derive(Debug, Deserialize, Validate)]
pub struct CreateCompanyRequest {
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    
    #[validate(length(min = 10, max = 500))]
    pub address: String,
    
    #[validate(length(min = 3, max = 50))]
    pub subscription_plan: String,
    
    #[validate(length(min = 3, max = 20))]
    pub subscription_status: String,
    
    #[validate(range(min = 1, max = 100))]
    pub max_drivers: i32,
    
    #[validate(range(min = 1, max = 50))]
    pub max_vehicles: i32,
}

/// Request para actualizar una company existente
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCompanyRequest {
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    
    #[validate(length(min = 10, max = 500))]
    pub address: Option<String>,
    
    #[validate(length(min = 3, max = 50))]
    pub subscription_plan: Option<String>,
    
    #[validate(length(min = 3, max = 20))]
    pub subscription_status: Option<String>,
    
    #[validate(range(min = 1, max = 100))]
    pub max_drivers: Option<i32>,
    
    #[validate(range(min = 1, max = 50))]
    pub max_vehicles: Option<i32>,
}

/// Response de company para la API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyResponse {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub subscription_plan: String,
    pub subscription_status: String,
    pub max_drivers: i32,
    pub max_vehicles: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response de company para listados paginados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyListResponse {
    pub companies: Vec<CompanyResponse>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

/// Filtros para búsqueda de companies
#[derive(Debug, Clone, Deserialize)]
pub struct CompanyFilters {
    pub name: Option<String>,
    pub subscription_plan: Option<String>,
    pub subscription_status: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub page: Option<i32>,
}

impl From<Company> for CompanyResponse {
    fn from(company: Company) -> Self {
        Self {
            id: company.id,
            name: company.name,
            address: company.address,
            subscription_plan: company.subscription_plan,
            subscription_status: company.subscription_status,
            max_drivers: company.max_drivers,
            max_vehicles: company.max_vehicles,
            created_at: company.created_at,
            updated_at: company.updated_at,
        }
    }
}