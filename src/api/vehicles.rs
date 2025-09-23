//! Handlers de Vehicles
//! 
//! Este módulo maneja las operaciones CRUD para vehículos.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::vehicle::{
        Vehicle, VehicleStatus, VehicleResponse, VehicleListResponse,
        CreateVehicleRequest, UpdateVehicleRequest, VehicleFilters,
    },
    utils::errors::{AppError, AppResult},
    middleware::auth::AuthenticatedUser,
};

/// Obtener todos los vehículos con filtros
pub async fn get_vehicles(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Query(filters): Query<VehicleFilters>,
) -> AppResult<Json<Vec<VehicleListResponse>>> {
    let limit = filters.limit.unwrap_or(50).min(100);
    let offset = filters.offset.unwrap_or(0);

    let rows = sqlx::query!(
        r#"
        SELECT 
            id, company_id, license_plate, brand, model, year, color,
            vehicle_status as "vehicle_status: String", current_mileage, fuel_type,
            fuel_capacity, weekly_fuel_allocation, total_damage_cost, 
            damage_incidents_count, vin, engine_size, transmission,
            created_at, updated_at, deleted_at
        FROM vehicles 
        WHERE company_id = $1 
        AND deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user.company_id,
        limit,
        offset
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let vehicles: Vec<VehicleListResponse> = rows
        .into_iter()
        .map(|row| {
            let vehicle = Vehicle {
                id: row.id,
                company_id: row.company_id,
                license_plate: row.license_plate,
                brand: row.brand,
                model: row.model,
                year: row.year,
                color: row.color,
                vehicle_status: match row.vehicle_status.as_str() {
                    "active" => VehicleStatus::Active,
                    "maintenance" => VehicleStatus::Maintenance,
                    "out_of_service" => VehicleStatus::OutOfService,
                    "retired" => VehicleStatus::Retired,
                    _ => VehicleStatus::Active,
                },
                current_mileage: row.current_mileage,
                fuel_type: row.fuel_type,
                fuel_capacity: row.fuel_capacity,
                weekly_fuel_allocation: row.weekly_fuel_allocation,
                total_damage_cost: row.total_damage_cost,
                damage_incidents_count: row.damage_incidents_count,
                vin: row.vin,
                engine_size: row.engine_size,
                transmission: row.transmission,
                created_at: row.created_at,
                updated_at: row.updated_at,
                deleted_at: row.deleted_at,
            };
            VehicleListResponse::from(vehicle)
        })
        .collect();

    Ok(Json(vehicles))
}

/// Obtener un vehículo por ID
pub async fn get_vehicle(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<VehicleResponse>> {
    let row = sqlx::query!(
        r#"
        SELECT 
            id, company_id, license_plate, brand, model, year, color,
            vehicle_status as "vehicle_status: String", current_mileage, fuel_type,
            fuel_capacity, weekly_fuel_allocation, total_damage_cost, 
            damage_incidents_count, vin, engine_size, transmission,
            created_at, updated_at, deleted_at
        FROM vehicles 
        WHERE id = $1 AND company_id = $2 AND deleted_at IS NULL
        "#,
        id,
        user.company_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound("Vehículo no encontrado".to_string()),
        _ => AppError::Database(e),
    })?;

    let vehicle = Vehicle {
        id: row.id,
        company_id: row.company_id,
        license_plate: row.license_plate,
        brand: row.brand,
        model: row.model,
        year: row.year,
        color: row.color,
        vehicle_status: match row.vehicle_status.as_str() {
            "active" => VehicleStatus::Active,
            "maintenance" => VehicleStatus::Maintenance,
            "out_of_service" => VehicleStatus::OutOfService,
            "retired" => VehicleStatus::Retired,
            _ => VehicleStatus::Active,
        },
        current_mileage: row.current_mileage,
        fuel_type: row.fuel_type,
        fuel_capacity: row.fuel_capacity,
        weekly_fuel_allocation: row.weekly_fuel_allocation,
        total_damage_cost: row.total_damage_cost,
        damage_incidents_count: row.damage_incidents_count,
        vin: row.vin,
        engine_size: row.engine_size,
        transmission: row.transmission,
        created_at: row.created_at,
        updated_at: row.updated_at,
        deleted_at: row.deleted_at,
    };

    Ok(Json(VehicleResponse::from(vehicle)))
}

/// Crear un nuevo vehículo
pub async fn create_vehicle(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Json(vehicle_data): Json<CreateVehicleRequest>,
) -> AppResult<Json<VehicleResponse>> {
    // Validar datos de entrada
    vehicle_data.validate()
        .map_err(AppError::Validation)?;

    let row = sqlx::query!(
        r#"
        INSERT INTO vehicles (
            company_id, license_plate, brand, model, year, color,
            vehicle_status, current_mileage, fuel_type, fuel_capacity,
            weekly_fuel_allocation, total_damage_cost, damage_incidents_count,
            vin, engine_size, transmission, created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, ($7::text)::vehicle_status, $8, $9, $10, $11, $12, $13, $14, $15, $16, NOW(), NOW()
        )
        RETURNING 
            id, company_id, license_plate, brand, model, year, color,
            vehicle_status as "vehicle_status: crate::models::vehicle::VehicleStatus", current_mileage, fuel_type,
            fuel_capacity, weekly_fuel_allocation, total_damage_cost, 
            damage_incidents_count, vin, engine_size, transmission,
            created_at, updated_at, deleted_at
        "#,
        user.company_id,
        vehicle_data.license_plate,
        vehicle_data.brand,
        vehicle_data.model,
        vehicle_data.year,
        vehicle_data.color,
        "active",
        rust_decimal::Decimal::new(0, 0), // current_mileage = 0
        vehicle_data.fuel_type,
        vehicle_data.fuel_capacity,
        vehicle_data.weekly_fuel_allocation,
        rust_decimal::Decimal::new(0, 0), // total_damage_cost = 0
        0, // damage_incidents_count = 0
        vehicle_data.vin,
        vehicle_data.engine_size,
        vehicle_data.transmission
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let vehicle = Vehicle {
        id: row.id,
        company_id: row.company_id,
        license_plate: row.license_plate,
        brand: row.brand,
        model: row.model,
        year: row.year,
        color: row.color,
        vehicle_status: row.vehicle_status,
        current_mileage: row.current_mileage,
        fuel_type: row.fuel_type,
        fuel_capacity: row.fuel_capacity,
        weekly_fuel_allocation: row.weekly_fuel_allocation,
        total_damage_cost: row.total_damage_cost,
        damage_incidents_count: row.damage_incidents_count,
        vin: row.vin,
        engine_size: row.engine_size,
        transmission: row.transmission,
        created_at: row.created_at,
        updated_at: row.updated_at,
        deleted_at: row.deleted_at,
    };

    Ok(Json(VehicleResponse::from(vehicle)))
}

/// Actualizar un vehículo existente
pub async fn update_vehicle(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Path(id): Path<Uuid>,
    Json(vehicle_data): Json<UpdateVehicleRequest>,
) -> AppResult<Json<VehicleResponse>> {
    // Validar datos de entrada
    vehicle_data.validate()
        .map_err(AppError::Validation)?;

    // Verificar que el vehículo existe y pertenece a la empresa
    let _existing = sqlx::query!(
        "SELECT id FROM vehicles WHERE id = $1 AND company_id = $2 AND deleted_at IS NULL",
        id,
        user.company_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound("Vehículo no encontrado".to_string()),
        _ => AppError::Database(e),
    })?;

    let row = sqlx::query!(
        r#"
        UPDATE vehicles SET
            license_plate = COALESCE($2, license_plate),
            brand = COALESCE($3, brand),
            model = COALESCE($4, model),
            year = COALESCE($5, year),
            color = COALESCE($6, color),
            vehicle_status = COALESCE(($7::text)::vehicle_status, vehicle_status),
            current_mileage = COALESCE($8, current_mileage),
            fuel_type = COALESCE($9, fuel_type),
            fuel_capacity = COALESCE($10, fuel_capacity),
            weekly_fuel_allocation = COALESCE($11, weekly_fuel_allocation),
            vin = COALESCE($12, vin),
            engine_size = COALESCE($13, engine_size),
            transmission = COALESCE($14, transmission),
            updated_at = NOW()
        WHERE id = $1 AND company_id = $15
        RETURNING 
            id, company_id, license_plate, brand, model, year, color,
            vehicle_status as "vehicle_status: crate::models::vehicle::VehicleStatus", current_mileage, fuel_type,
            fuel_capacity, weekly_fuel_allocation, total_damage_cost, 
            damage_incidents_count, vin, engine_size, transmission,
            created_at, updated_at, deleted_at
        "#,
        id,
        vehicle_data.license_plate,
        vehicle_data.brand,
        vehicle_data.model,
        vehicle_data.year,
        vehicle_data.color,
        vehicle_data.vehicle_status,
        vehicle_data.current_mileage,
        vehicle_data.fuel_type,
        vehicle_data.fuel_capacity,
        vehicle_data.weekly_fuel_allocation,
        vehicle_data.vin,
        vehicle_data.engine_size,
        vehicle_data.transmission,
        user.company_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let vehicle = Vehicle {
        id: row.id,
        company_id: row.company_id,
        license_plate: row.license_plate,
        brand: row.brand,
        model: row.model,
        year: row.year,
        color: row.color,
        vehicle_status: row.vehicle_status,
        current_mileage: row.current_mileage,
        fuel_type: row.fuel_type,
        fuel_capacity: row.fuel_capacity,
        weekly_fuel_allocation: row.weekly_fuel_allocation,
        total_damage_cost: row.total_damage_cost,
        damage_incidents_count: row.damage_incidents_count,
        vin: row.vin,
        engine_size: row.engine_size,
        transmission: row.transmission,
        created_at: row.created_at,
        updated_at: row.updated_at,
        deleted_at: row.deleted_at,
    };

    Ok(Json(VehicleResponse::from(vehicle)))
}

/// Eliminar un vehículo (soft delete)
pub async fn delete_vehicle(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let result = sqlx::query!(
        r#"
        UPDATE vehicles 
        SET deleted_at = NOW(), updated_at = NOW()
        WHERE id = $1 AND company_id = $2 AND deleted_at IS NULL
        "#,
        id,
        user.company_id
    )
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Vehículo no encontrado".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}