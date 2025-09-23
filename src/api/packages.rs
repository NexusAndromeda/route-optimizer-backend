//! Handlers de Packages
//! 
//! Este módulo maneja las operaciones CRUD para paquetes.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::package::{
        Package, DeliveryStatus, DeliveryFailureReason, PackageResponse, PackageListResponse,
        CreatePackageRequest, UpdatePackageRequest, PackageFilters,
        MarkDeliveredRequest, MarkFailedRequest,
    },
    utils::errors::{AppError, AppResult},
    middleware::auth::AuthenticatedUser,
};

/// Obtener todos los paquetes con filtros
pub async fn get_packages(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Query(filters): Query<PackageFilters>,
) -> AppResult<Json<Vec<PackageListResponse>>> {
    let limit = filters.limit.unwrap_or(50).min(100);
    let offset = filters.offset.unwrap_or(0);

    let rows = sqlx::query!(
        r#"
        SELECT 
            id, company_id, tournee_id, tracking_number, external_tracking_number,
            package_origin, external_package_id, delivery_status as "delivery_status: String",
            delivery_date, delivery_attempts, recipient_name, delivery_address,
            failure_reason as "failure_reason: String", created_at, updated_at, deleted_at
        FROM packages 
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

    let packages: Vec<PackageListResponse> = rows
        .into_iter()
        .map(|row| {
            let package = Package {
                id: row.id,
                company_id: row.company_id,
                tournee_id: row.tournee_id,
                tracking_number: row.tracking_number,
                external_tracking_number: row.external_tracking_number,
                package_origin: row.package_origin,
                external_package_id: row.external_package_id,
                integration_id: None,
                package_type: None,
                package_weight: None,
                package_dimensions: None,
                delivery_status: match row.delivery_status.as_str() {
                    "pending" => DeliveryStatus::Pending,
                    "intransit" => DeliveryStatus::InTransit,
                    "outfordelivery" => DeliveryStatus::OutForDelivery,
                    "delivered" => DeliveryStatus::Delivered,
                    "failed" => DeliveryStatus::Failed,
                    "returned" => DeliveryStatus::Returned,
                    "cancelled" => DeliveryStatus::Cancelled,
                    _ => DeliveryStatus::Pending,
                },
                delivery_date: row.delivery_date,
                delivery_time: None,
                delivery_attempts: row.delivery_attempts.expect("delivery_attempts should not be null"),
                recipient_name: row.recipient_name,
                recipient_phone: None,
                delivery_address: row.delivery_address,
                delivery_instructions: None,
                failure_reason: row.failure_reason.map(|r| match r.as_str() {
                    "recipientnothome" => DeliveryFailureReason::RecipientNotHome,
                    "wrongaddress" => DeliveryFailureReason::WrongAddress,
                    "packagedamaged" => DeliveryFailureReason::PackageDamaged,
                    "refuseddelivery" => DeliveryFailureReason::RefusedDelivery,
                    "securityrestriction" => DeliveryFailureReason::SecurityRestriction,
                    "weatherconditions" => DeliveryFailureReason::WeatherConditions,
                    "vehiclebreakdown" => DeliveryFailureReason::VehicleBreakdown,
                    "driveremergency" => DeliveryFailureReason::DriverEmergency,
                    _ => DeliveryFailureReason::RecipientNotHome,
                }),
                failure_notes: None,
                reschedule_date: None,
                delivery_photo: None,
                signature_required: false,
                signature_image: None,
                signature_photo: None,
                delivery_coordinates: None,
                delivery_duration_minutes: None,
                driver_notes: None,
                package_condition: None,
                created_at: row.created_at,
                updated_at: row.updated_at,
                deleted_at: row.deleted_at,
            };
            PackageListResponse::from(package)
        })
        .collect();

    Ok(Json(packages))
}

/// Obtener un paquete por ID
pub async fn get_package(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<PackageResponse>> {
    let row = sqlx::query!(
        r#"
        SELECT 
            id, company_id, tournee_id, tracking_number, external_tracking_number,
            package_origin, external_package_id, integration_id, package_type,
            package_weight, package_dimensions, delivery_status as "delivery_status: crate::models::package::DeliveryStatus",
            delivery_date, delivery_time, delivery_attempts, recipient_name,
            recipient_phone, delivery_address, delivery_instructions,
            failure_reason as "failure_reason: crate::models::package::DeliveryFailureReason", failure_notes, reschedule_date,
            delivery_photo, signature_required, signature_image, signature_photo,
            delivery_duration_minutes, driver_notes, package_condition,
            created_at, updated_at, deleted_at
        FROM packages 
        WHERE id = $1 AND company_id = $2 AND deleted_at IS NULL
        "#,
        id,
        user.company_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound("Paquete no encontrado".to_string()),
        _ => AppError::Database(e),
    })?;

    let package = Package {
        id: row.id,
        company_id: row.company_id,
        tournee_id: row.tournee_id,
        tracking_number: row.tracking_number,
        external_tracking_number: row.external_tracking_number,
        package_origin: row.package_origin,
        external_package_id: row.external_package_id,
        integration_id: row.integration_id,
        package_type: row.package_type,
        package_weight: row.package_weight,
        package_dimensions: row.package_dimensions,
        delivery_status: row.delivery_status,
        delivery_date: row.delivery_date,
        delivery_time: row.delivery_time,
        delivery_attempts: row.delivery_attempts.expect("delivery_attempts should not be null"),
        recipient_name: row.recipient_name,
        recipient_phone: row.recipient_phone,
        delivery_address: row.delivery_address,
        delivery_instructions: row.delivery_instructions,
        failure_reason: row.failure_reason,
        failure_notes: row.failure_notes,
        reschedule_date: row.reschedule_date,
        delivery_photo: row.delivery_photo,
        signature_required: row.signature_required.expect("signature_required should not be null"),
        signature_image: row.signature_image,
        signature_photo: row.signature_photo,
        delivery_coordinates: None,
        delivery_duration_minutes: row.delivery_duration_minutes,
        driver_notes: row.driver_notes,
        package_condition: row.package_condition,
        created_at: row.created_at,
        updated_at: row.updated_at,
        deleted_at: row.deleted_at,
    };

    Ok(Json(PackageResponse::from(package)))
}

/// Crear un nuevo paquete
pub async fn create_package(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Json(package_data): Json<CreatePackageRequest>,
) -> AppResult<Json<PackageResponse>> {
    // Validar datos de entrada
    package_data.validate()
        .map_err(AppError::Validation)?;

    let tournee_id = Uuid::parse_str(&package_data.tournee_id)
        .map_err(|_| AppError::BadRequest("ID de tournée inválido".to_string()))?;

    let row = sqlx::query!(
        r#"
        INSERT INTO packages (
            company_id, tournee_id, tracking_number, external_tracking_number,
            package_origin, external_package_id, package_type, package_weight,
            package_dimensions, delivery_status, delivery_attempts,
            recipient_name, recipient_phone, delivery_address, delivery_instructions,
            signature_required, created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, ($10::text)::delivery_status, $11, $12, $13, $14, $15, $16, NOW(), NOW()
        )
        RETURNING 
            id, company_id, tournee_id, tracking_number, external_tracking_number,
            package_origin, external_package_id, integration_id, package_type,
            package_weight, package_dimensions, delivery_status as "delivery_status: crate::models::package::DeliveryStatus",
            delivery_date, delivery_time, delivery_attempts, recipient_name,
            recipient_phone, delivery_address, delivery_instructions,
            failure_reason as "failure_reason: crate::models::package::DeliveryFailureReason", failure_notes, reschedule_date,
            delivery_photo, signature_required, signature_image, signature_photo,
            delivery_duration_minutes, driver_notes, package_condition,
            created_at, updated_at, deleted_at
        "#,
        user.company_id,
        tournee_id,
        package_data.tracking_number,
        package_data.external_tracking_number,
        package_data.package_origin.unwrap_or_else(|| "manual".to_string()),
        package_data.external_package_id,
        package_data.package_type,
        package_data.package_weight,
        package_data.package_dimensions,
        "pending",
        0, // delivery_attempts = 0
        package_data.recipient_name,
        package_data.recipient_phone,
        package_data.delivery_address,
        package_data.delivery_instructions,
        package_data.signature_required.unwrap_or(false)
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let package = Package {
        id: row.id,
        company_id: row.company_id,
        tournee_id: row.tournee_id,
        tracking_number: row.tracking_number,
        external_tracking_number: row.external_tracking_number,
        package_origin: row.package_origin,
        external_package_id: row.external_package_id,
        integration_id: row.integration_id,
        package_type: row.package_type,
        package_weight: row.package_weight,
        package_dimensions: row.package_dimensions,
        delivery_status: row.delivery_status,
        delivery_date: row.delivery_date,
        delivery_time: row.delivery_time,
        delivery_attempts: row.delivery_attempts.expect("delivery_attempts should not be null"),
        recipient_name: row.recipient_name,
        recipient_phone: row.recipient_phone,
        delivery_address: row.delivery_address,
        delivery_instructions: row.delivery_instructions,
        failure_reason: row.failure_reason,
        failure_notes: row.failure_notes,
        reschedule_date: row.reschedule_date,
        delivery_photo: row.delivery_photo,
        signature_required: row.signature_required.expect("signature_required should not be null"),
        signature_image: row.signature_image,
        signature_photo: row.signature_photo,
        delivery_coordinates: None,
        delivery_duration_minutes: row.delivery_duration_minutes,
        driver_notes: row.driver_notes,
        package_condition: row.package_condition,
        created_at: row.created_at,
        updated_at: row.updated_at,
        deleted_at: row.deleted_at,
    };

    Ok(Json(PackageResponse::from(package)))
}

/// Marcar paquete como entregado
pub async fn mark_delivered(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Path(id): Path<Uuid>,
    Json(delivery_data): Json<MarkDeliveredRequest>,
) -> AppResult<Json<PackageResponse>> {
    // Validar datos de entrada
    delivery_data.validate()
        .map_err(AppError::Validation)?;

    let row = sqlx::query!(
        r#"
        UPDATE packages SET
            delivery_status = 'delivered',
            delivery_date = CURRENT_DATE,
            delivery_time = CURRENT_TIME,
            delivery_photo = $2,
            signature_image = $3,
            signature_photo = $4,
            delivery_duration_minutes = $5,
            driver_notes = $6,
            package_condition = $7,
            updated_at = NOW()
        WHERE id = $1 AND company_id = $8
        RETURNING 
            id, company_id, tournee_id, tracking_number, external_tracking_number,
            package_origin, external_package_id, integration_id, package_type,
            package_weight, package_dimensions, delivery_status as "delivery_status: crate::models::package::DeliveryStatus",
            delivery_date, delivery_time, delivery_attempts, recipient_name,
            recipient_phone, delivery_address, delivery_instructions,
            failure_reason as "failure_reason: crate::models::package::DeliveryFailureReason", failure_notes, reschedule_date,
            delivery_photo, signature_required, signature_image, signature_photo,
            delivery_duration_minutes, driver_notes, package_condition,
            created_at, updated_at, deleted_at
        "#,
        id,
        delivery_data.delivery_photo,
        delivery_data.signature_image,
        delivery_data.signature_photo,
        delivery_data.delivery_duration_minutes,
        delivery_data.driver_notes,
        delivery_data.package_condition,
        user.company_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound("Paquete no encontrado".to_string()),
        _ => AppError::Database(e),
    })?;

    let package = Package {
        id: row.id,
        company_id: row.company_id,
        tournee_id: row.tournee_id,
        tracking_number: row.tracking_number,
        external_tracking_number: row.external_tracking_number,
        package_origin: row.package_origin,
        external_package_id: row.external_package_id,
        integration_id: row.integration_id,
        package_type: row.package_type,
        package_weight: row.package_weight,
        package_dimensions: row.package_dimensions,
        delivery_status: row.delivery_status,
        delivery_date: row.delivery_date,
        delivery_time: row.delivery_time,
        delivery_attempts: row.delivery_attempts.expect("delivery_attempts should not be null"),
        recipient_name: row.recipient_name,
        recipient_phone: row.recipient_phone,
        delivery_address: row.delivery_address,
        delivery_instructions: row.delivery_instructions,
        failure_reason: None,
        failure_notes: row.failure_notes,
        reschedule_date: row.reschedule_date,
        delivery_photo: row.delivery_photo,
        signature_required: row.signature_required.expect("signature_required should not be null"),
        signature_image: row.signature_image,
        signature_photo: row.signature_photo,
        delivery_coordinates: None,
        delivery_duration_minutes: row.delivery_duration_minutes,
        driver_notes: row.driver_notes,
        package_condition: row.package_condition,
        created_at: row.created_at,
        updated_at: row.updated_at,
        deleted_at: row.deleted_at,
    };

    Ok(Json(PackageResponse::from(package)))
}

/// Marcar paquete como fallido
pub async fn mark_failed(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Path(id): Path<Uuid>,
    Json(failure_data): Json<MarkFailedRequest>,
) -> AppResult<Json<PackageResponse>> {
    // Validar datos de entrada
    failure_data.validate()
        .map_err(AppError::Validation)?;

    let failure_reason_enum = match failure_data.failure_reason.as_str() {
        "recipient_not_home" => "recipient_not_home",
        "wrong_address" => "wrong_address",
        "package_damaged" => "package_damaged",
        "refused_delivery" => "refused_delivery",
        "security_restriction" => "security_restriction",
        "weather_conditions" => "weather_conditions",
        "vehicle_breakdown" => "vehicle_breakdown",
        "driver_emergency" => "driver_emergency",
        _ => "recipient_not_home",
    };

    let reschedule_date = if let Some(date_str) = &failure_data.reschedule_date {
        chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
    } else {
        None
    };

    let row = sqlx::query!(
        r#"
        UPDATE packages SET
            delivery_status = 'failed',
            failure_reason = ($2::text)::delivery_failure_reason,
            failure_notes = $3,
            reschedule_date = $4,
            driver_notes = $5,
            delivery_attempts = delivery_attempts + 1,
            updated_at = NOW()
        WHERE id = $1 AND company_id = $6
        RETURNING 
            id, company_id, tournee_id, tracking_number, external_tracking_number,
            package_origin, external_package_id, integration_id, package_type,
            package_weight, package_dimensions, delivery_status as "delivery_status: crate::models::package::DeliveryStatus",
            delivery_date, delivery_time, delivery_attempts, recipient_name,
            recipient_phone, delivery_address, delivery_instructions,
            failure_reason as "failure_reason: crate::models::package::DeliveryFailureReason", failure_notes, reschedule_date,
            delivery_photo, signature_required, signature_image, signature_photo,
            delivery_duration_minutes, driver_notes, package_condition,
            created_at, updated_at, deleted_at
        "#,
        id,
        failure_reason_enum,
        failure_data.failure_notes,
        reschedule_date,
        failure_data.driver_notes,
        user.company_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound("Paquete no encontrado".to_string()),
        _ => AppError::Database(e),
    })?;

    let package = Package {
        id: row.id,
        company_id: row.company_id,
        tournee_id: row.tournee_id,
        tracking_number: row.tracking_number,
        external_tracking_number: row.external_tracking_number,
        package_origin: row.package_origin,
        external_package_id: row.external_package_id,
        integration_id: row.integration_id,
        package_type: row.package_type,
        package_weight: row.package_weight,
        package_dimensions: row.package_dimensions,
        delivery_status: row.delivery_status,
        delivery_date: row.delivery_date,
        delivery_time: row.delivery_time,
        delivery_attempts: row.delivery_attempts.expect("delivery_attempts should not be null"),
        recipient_name: row.recipient_name,
        recipient_phone: row.recipient_phone,
        delivery_address: row.delivery_address,
        delivery_instructions: row.delivery_instructions,
        failure_reason: row.failure_reason,
        failure_notes: row.failure_notes,
        reschedule_date: row.reschedule_date,
        delivery_photo: row.delivery_photo,
        signature_required: row.signature_required.expect("signature_required should not be null"),
        signature_image: row.signature_image,
        signature_photo: row.signature_photo,
        delivery_coordinates: None,
        delivery_duration_minutes: row.delivery_duration_minutes,
        driver_notes: row.driver_notes,
        package_condition: row.package_condition,
        created_at: row.created_at,
        updated_at: row.updated_at,
        deleted_at: row.deleted_at,
    };

    Ok(Json(PackageResponse::from(package)))
}

/// Eliminar un paquete (soft delete)
pub async fn delete_package(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let result = sqlx::query!(
        r#"
        UPDATE packages 
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
        return Err(AppError::NotFound("Paquete no encontrado".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}