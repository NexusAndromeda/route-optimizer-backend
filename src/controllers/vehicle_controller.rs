use crate::dto::vehicle_dto::{CreateVehicleRequest, UpdateVehicleRequest, VehicleResponse};
use crate::dto::company_dto::ApiResponse;
use crate::repositories::vehicle_repository::VehicleRepository;
use crate::utils::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;

pub struct VehicleController {
    repository: VehicleRepository,
}

impl VehicleController {
    pub fn new(pool: PgPool) -> Self {
        Self {
            repository: VehicleRepository::new(pool),
        }
    }

    pub async fn create(
        &self,
        company_id: Uuid,
        request: CreateVehicleRequest,
    ) -> Result<ApiResponse<VehicleResponse>, AppError> {
        // Validar campos
        if request.license_plate.trim().is_empty() {
            return Err(AppError::ValidationError("La matrícula es requerida".to_string()));
        }

        // Verificar que la matrícula no exista para esta empresa
        if self.repository.license_plate_exists(&request.license_plate, company_id).await? {
            return Err(AppError::Conflict("La matrícula ya está registrada para esta empresa".to_string()));
        }

        // Crear vehículo
        let vehicle = self.repository.create(
            company_id,
            request.license_plate,
            request.brand,
            request.model,
            request.fuel_type.unwrap_or_else(|| "diesel".to_string()),
            request.current_mileage.unwrap_or(0.0),
        ).await?;

        // Convertir a DTO
        let response = VehicleResponse {
            id: vehicle.id,
            company_id: vehicle.company_id,
            license_plate: vehicle.license_plate,
            brand: vehicle.brand,
            model: vehicle.model,
            vehicle_status: vehicle.vehicle_status,
            current_mileage: vehicle.current_mileage.to_string().parse().unwrap_or(0.0),
            fuel_type: vehicle.fuel_type,
            created_at: vehicle.created_at,
        };

        Ok(ApiResponse::success_with_message(
            response,
            "Vehículo creado exitosamente".to_string()
        ))
    }

    pub async fn get_by_id(
        &self,
        id: Uuid,
        company_id: Uuid,
    ) -> Result<VehicleResponse, AppError> {
        let vehicle = self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Vehículo no encontrado".to_string()))?;

        // Verificar que pertenece a la empresa
        if vehicle.company_id != company_id {
            return Err(AppError::Forbidden("No tienes permiso para acceder a este vehículo".to_string()));
        }

        Ok(VehicleResponse {
            id: vehicle.id,
            company_id: vehicle.company_id,
            license_plate: vehicle.license_plate,
            brand: vehicle.brand,
            model: vehicle.model,
            vehicle_status: vehicle.vehicle_status,
            current_mileage: vehicle.current_mileage.to_string().parse().unwrap_or(0.0),
            fuel_type: vehicle.fuel_type,
            created_at: vehicle.created_at,
        })
    }

    pub async fn list_by_company(
        &self,
        company_id: Uuid,
    ) -> Result<Vec<VehicleResponse>, AppError> {
        let vehicles = self.repository.find_by_company(company_id).await?;

        let response = vehicles.into_iter().map(|v| VehicleResponse {
            id: v.id,
            company_id: v.company_id,
            license_plate: v.license_plate,
            brand: v.brand,
            model: v.model,
            vehicle_status: v.vehicle_status,
            current_mileage: v.current_mileage.to_string().parse().unwrap_or(0.0),
            fuel_type: v.fuel_type,
            created_at: v.created_at,
        }).collect();

        Ok(response)
    }

    pub async fn update(
        &self,
        id: Uuid,
        company_id: Uuid,
        request: UpdateVehicleRequest,
    ) -> Result<ApiResponse<VehicleResponse>, AppError> {
        let vehicle = self.repository.update(
            id,
            company_id,
            request.license_plate,
            request.brand,
            request.model,
            request.vehicle_status,
            request.current_mileage,
            request.fuel_type,
        ).await?;

        let response = VehicleResponse {
            id: vehicle.id,
            company_id: vehicle.company_id,
            license_plate: vehicle.license_plate,
            brand: vehicle.brand,
            model: vehicle.model,
            vehicle_status: vehicle.vehicle_status,
            current_mileage: vehicle.current_mileage.to_string().parse().unwrap_or(0.0),
            fuel_type: vehicle.fuel_type,
            created_at: vehicle.created_at,
        };

        Ok(ApiResponse::success_with_message(
            response,
            "Vehículo actualizado exitosamente".to_string()
        ))
    }

    pub async fn delete(
        &self,
        id: Uuid,
        company_id: Uuid,
    ) -> Result<(), AppError> {
        self.repository.delete(id, company_id).await?;
        Ok(())
    }
}
