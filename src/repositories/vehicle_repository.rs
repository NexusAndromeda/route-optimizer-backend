use crate::utils::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

// Struct simplificado para Vehicle
#[derive(Debug, sqlx::FromRow)]
pub struct Vehicle {
    pub id: Uuid,
    pub company_id: Uuid,
    pub license_plate: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub vehicle_status: String,
    pub current_mileage: sqlx::types::Decimal,
    pub fuel_type: String,
    pub created_at: chrono::DateTime<Utc>,
}

pub struct VehicleRepository {
    pool: PgPool,
}

impl VehicleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        company_id: Uuid,
        license_plate: String,
        brand: Option<String>,
        model: Option<String>,
        fuel_type: String,
        current_mileage: f64,
    ) -> Result<Vehicle, AppError> {
        let id = Uuid::new_v4();
        let mileage = sqlx::types::Decimal::from_f64_retain(current_mileage)
            .ok_or_else(|| AppError::ValidationError("Invalid mileage value".to_string()))?;

        let vehicle = sqlx::query_as::<_, Vehicle>(
            r#"
            INSERT INTO vehicles (id, company_id, license_plate, brand, model, vehicle_status, current_mileage, fuel_type, created_at)
            VALUES ($1, $2, $3, $4, $5, 'active', $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(company_id)
        .bind(license_plate)
        .bind(brand)
        .bind(model)
        .bind(mileage)
        .bind(fuel_type)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Error creating vehicle: {}", e)))?;

        Ok(vehicle)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Vehicle>, AppError> {
        let vehicle = sqlx::query_as::<_, Vehicle>("SELECT * FROM vehicles WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Error finding vehicle: {}", e)))?;

        Ok(vehicle)
    }

    pub async fn find_by_company(&self, company_id: Uuid) -> Result<Vec<Vehicle>, AppError> {
        let vehicles = sqlx::query_as::<_, Vehicle>(
            "SELECT * FROM vehicles WHERE company_id = $1 ORDER BY created_at DESC"
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Error listing vehicles: {}", e)))?;

        Ok(vehicles)
    }

    pub async fn license_plate_exists(&self, license_plate: &str, company_id: Uuid) -> Result<bool, AppError> {
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM vehicles WHERE license_plate = $1 AND company_id = $2)"
        )
        .bind(license_plate)
        .bind(company_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Error checking license plate: {}", e)))?;

        Ok(result.0)
    }

    pub async fn update(
        &self,
        id: Uuid,
        company_id: Uuid,
        license_plate: Option<String>,
        brand: Option<String>,
        model: Option<String>,
        vehicle_status: Option<String>,
        current_mileage: Option<f64>,
        fuel_type: Option<String>,
    ) -> Result<Vehicle, AppError> {
        // Obtener vehículo actual
        let current = self.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("Vehicle not found".to_string()))?;

        // Verificar que pertenece a la empresa
        if current.company_id != company_id {
            return Err(AppError::Forbidden("Vehicle does not belong to this company".to_string()));
        }

        let mileage = if let Some(m) = current_mileage {
            sqlx::types::Decimal::from_f64_retain(m)
                .ok_or_else(|| AppError::ValidationError("Invalid mileage value".to_string()))?
        } else {
            current.current_mileage
        };

        let vehicle = sqlx::query_as::<_, Vehicle>(
            r#"
            UPDATE vehicles
            SET license_plate = $2, brand = $3, model = $4, vehicle_status = $5, current_mileage = $6, fuel_type = $7
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(license_plate.unwrap_or(current.license_plate))
        .bind(brand.or(current.brand))
        .bind(model.or(current.model))
        .bind(vehicle_status.unwrap_or(current.vehicle_status))
        .bind(mileage)
        .bind(fuel_type.unwrap_or(current.fuel_type))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Error updating vehicle: {}", e)))?;

        Ok(vehicle)
    }

    pub async fn delete(&self, id: Uuid, company_id: Uuid) -> Result<(), AppError> {
        // Verificar que pertenece a la empresa
        let vehicle = self.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("Vehicle not found".to_string()))?;

        if vehicle.company_id != company_id {
            return Err(AppError::Forbidden("Vehicle does not belong to this company".to_string()));
        }

        sqlx::query("DELETE FROM vehicles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Error deleting vehicle: {}", e)))?;

        Ok(())
    }
}
