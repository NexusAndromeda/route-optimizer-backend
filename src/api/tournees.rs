//! Handlers de Tournees
//! 
//! Este módulo maneja las operaciones CRUD para tournées.

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
    models::tournee::{
        Tournee, TourneeStatus, TourneeResponse, TourneeListResponse,
        CreateTourneeRequest, UpdateTourneeRequest, TourneeFilters,
        StartTourneeRequest, EndTourneeRequest,
    },
    utils::errors::{AppError, AppResult},
    middleware::auth::AuthenticatedUser,
};

/// Obtener todas las tournées con filtros
pub async fn get_tournees(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Query(filters): Query<TourneeFilters>,
) -> AppResult<Json<Vec<TourneeListResponse>>> {
    let limit = filters.limit.unwrap_or(50).min(100);
    let offset = filters.offset.unwrap_or(0);

    let rows = sqlx::query!(
        r#"
        SELECT 
            id, company_id, driver_id, vehicle_id, tournee_date, tournee_number,
            start_location, end_location, tournee_status as "tournee_status: String",
            start_time, end_time, start_mileage, end_mileage, total_distance,
            fuel_consumed, fuel_cost, tournee_origin, external_tournee_id,
            created_at, updated_at, deleted_at
        FROM tournees 
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

    let tournees: Vec<TourneeListResponse> = rows
        .into_iter()
        .map(|row| {
            let tournee = Tournee {
                id: row.id,
                company_id: row.company_id,
                driver_id: row.driver_id,
                vehicle_id: row.vehicle_id,
                tournee_date: row.tournee_date,
                tournee_number: row.tournee_number,
                start_location: row.start_location,
                end_location: row.end_location,
                tournee_status: match row.tournee_status.as_str() {
                    "pending" => TourneeStatus::Pending,
                    "in_progress" => TourneeStatus::InProgress,
                    "completed" => TourneeStatus::Completed,
                    "cancelled" => TourneeStatus::Cancelled,
                    "paused" => TourneeStatus::Paused,
                    _ => TourneeStatus::Pending,
                },
                start_time: row.start_time,
                end_time: row.end_time,
                start_mileage: row.start_mileage,
                end_mileage: row.end_mileage,
                total_distance: row.total_distance,
                fuel_consumed: row.fuel_consumed,
                fuel_cost: row.fuel_cost,
                pre_inspection_notes: None,
                post_inspection_notes: None,
                pre_inspection_photos: None,
                post_inspection_photos: None,
                route_optimization_score: None,
                estimated_duration_minutes: None,
                actual_duration_minutes: None,
                route_coordinates: None,
                traffic_conditions: None,
                weather_conditions: None,
                tournee_origin: row.tournee_origin,
                external_tournee_id: row.external_tournee_id,
                integration_id: None,
                created_at: row.created_at,
                updated_at: row.updated_at,
                deleted_at: row.deleted_at,
            };
            TourneeListResponse::from(tournee)
        })
        .collect();

    Ok(Json(tournees))
}

/// Obtener una tournée por ID
pub async fn get_tournee(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<TourneeResponse>> {
    let row = sqlx::query!(
        r#"
        SELECT 
            id, company_id, driver_id, vehicle_id, tournee_date, tournee_number,
            start_location, end_location, tournee_status as "tournee_status: String",
            start_time, end_time, start_mileage, end_mileage, total_distance,
            fuel_consumed, fuel_cost, pre_inspection_notes, post_inspection_notes,
            route_optimization_score, estimated_duration_minutes, actual_duration_minutes,
            tournee_origin, external_tournee_id, integration_id,
            created_at, updated_at, deleted_at
        FROM tournees 
        WHERE id = $1 AND company_id = $2 AND deleted_at IS NULL
        "#,
        id,
        user.company_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound("Tournée no encontrada".to_string()),
        _ => AppError::Database(e),
    })?;

    let tournee = Tournee {
        id: row.id,
        company_id: row.company_id,
        driver_id: row.driver_id,
        vehicle_id: row.vehicle_id,
        tournee_date: row.tournee_date,
        tournee_number: row.tournee_number,
        start_location: row.start_location,
        end_location: row.end_location,
        tournee_status: match row.tournee_status.as_str() {
            "pending" => TourneeStatus::Pending,
            "in_progress" => TourneeStatus::InProgress,
            "completed" => TourneeStatus::Completed,
            "cancelled" => TourneeStatus::Cancelled,
            "paused" => TourneeStatus::Paused,
            _ => TourneeStatus::Pending,
        },
        start_time: row.start_time,
        end_time: row.end_time,
        start_mileage: row.start_mileage,
        end_mileage: row.end_mileage,
        total_distance: row.total_distance,
        fuel_consumed: row.fuel_consumed,
        fuel_cost: row.fuel_cost,
        pre_inspection_notes: row.pre_inspection_notes,
        post_inspection_notes: row.post_inspection_notes,
        pre_inspection_photos: None,
        post_inspection_photos: None,
        route_optimization_score: row.route_optimization_score,
        estimated_duration_minutes: row.estimated_duration_minutes,
        actual_duration_minutes: row.actual_duration_minutes,
        route_coordinates: None,
        traffic_conditions: None,
        weather_conditions: None,
        tournee_origin: row.tournee_origin,
        external_tournee_id: row.external_tournee_id,
        integration_id: row.integration_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        deleted_at: row.deleted_at,
    };

    Ok(Json(TourneeResponse::from(tournee)))
}

/// Crear una nueva tournée
pub async fn create_tournee(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Json(tournee_data): Json<CreateTourneeRequest>,
) -> AppResult<Json<TourneeResponse>> {
    // Validar datos de entrada
    tournee_data.validate()
        .map_err(AppError::Validation)?;

    let driver_id = Uuid::parse_str(&tournee_data.driver_id)
        .map_err(|_| AppError::BadRequest("ID de conductor inválido".to_string()))?;
    let vehicle_id = Uuid::parse_str(&tournee_data.vehicle_id)
        .map_err(|_| AppError::BadRequest("ID de vehículo inválido".to_string()))?;

    let row = sqlx::query!(
        r#"
        INSERT INTO tournees (
            company_id, driver_id, vehicle_id, tournee_date, tournee_number,
            start_location, end_location, tournee_status, start_mileage,
            estimated_duration_minutes, tournee_origin, external_tournee_id,
            created_at, updated_at
        ) VALUES (
            $1, $2, $3, CURRENT_DATE, $4, $5, $6, ($7::text)::tournee_status, $8, $9, $10, $11, NOW(), NOW()
        )
        RETURNING 
            id, company_id, driver_id, vehicle_id, tournee_date, tournee_number,
            start_location, end_location, tournee_status as "tournee_status: crate::models::tournee::TourneeStatus",
            start_time, end_time, start_mileage, end_mileage, total_distance,
            fuel_consumed, fuel_cost, pre_inspection_notes, post_inspection_notes,
            route_optimization_score, estimated_duration_minutes, actual_duration_minutes,
            tournee_origin, external_tournee_id, integration_id,
            created_at, updated_at, deleted_at
        "#,
        user.company_id,
        driver_id,
        vehicle_id,
        tournee_data.tournee_number,
        tournee_data.start_location,
        tournee_data.end_location,
        "pending",
        tournee_data.start_mileage,
        tournee_data.estimated_duration_minutes,
        tournee_data.tournee_origin.unwrap_or_else(|| "manual".to_string()),
        tournee_data.external_tournee_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let tournee = Tournee {
        id: row.id,
        company_id: row.company_id,
        driver_id: row.driver_id,
        vehicle_id: row.vehicle_id,
        tournee_date: row.tournee_date,
        tournee_number: row.tournee_number,
        start_location: row.start_location,
        end_location: row.end_location,
        tournee_status: row.tournee_status,
        start_time: row.start_time,
        end_time: row.end_time,
        start_mileage: row.start_mileage,
        end_mileage: row.end_mileage,
        total_distance: row.total_distance,
        fuel_consumed: row.fuel_consumed,
        fuel_cost: row.fuel_cost,
        pre_inspection_notes: row.pre_inspection_notes,
        post_inspection_notes: row.post_inspection_notes,
        pre_inspection_photos: None,
        post_inspection_photos: None,
        route_optimization_score: row.route_optimization_score,
        estimated_duration_minutes: row.estimated_duration_minutes,
        actual_duration_minutes: row.actual_duration_minutes,
        route_coordinates: None,
        traffic_conditions: None,
        weather_conditions: None,
        tournee_origin: row.tournee_origin,
        external_tournee_id: row.external_tournee_id,
        integration_id: row.integration_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        deleted_at: row.deleted_at,
    };

    Ok(Json(TourneeResponse::from(tournee)))
}

/// Iniciar una tournée
pub async fn start_tournee(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Path(id): Path<Uuid>,
    Json(start_data): Json<StartTourneeRequest>,
) -> AppResult<Json<TourneeResponse>> {
    // Validar datos de entrada
    start_data.validate()
        .map_err(AppError::Validation)?;

    let row = sqlx::query!(
        r#"
        UPDATE tournees SET
            tournee_status = 'in_progress',
            start_time = NOW(),
            start_mileage = $2,
            pre_inspection_notes = $3,
            updated_at = NOW()
        WHERE id = $1 AND company_id = $4 AND tournee_status = 'pending'
        RETURNING 
            id, company_id, driver_id, vehicle_id, tournee_date, tournee_number,
            start_location, end_location, tournee_status as "tournee_status: String",
            start_time, end_time, start_mileage, end_mileage, total_distance,
            fuel_consumed, fuel_cost, pre_inspection_notes, post_inspection_notes,
            route_optimization_score, estimated_duration_minutes, actual_duration_minutes,
            tournee_origin, external_tournee_id, integration_id,
            created_at, updated_at, deleted_at
        "#,
        id,
        start_data.start_mileage,
        start_data.pre_inspection_notes,
        user.company_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound("Tournée no encontrada o ya iniciada".to_string()),
        _ => AppError::Database(e),
    })?;

    let tournee = Tournee {
        id: row.id,
        company_id: row.company_id,
        driver_id: row.driver_id,
        vehicle_id: row.vehicle_id,
        tournee_date: row.tournee_date,
        tournee_number: row.tournee_number,
        start_location: row.start_location,
        end_location: row.end_location,
        tournee_status: TourneeStatus::InProgress,
        start_time: row.start_time,
        end_time: row.end_time,
        start_mileage: row.start_mileage,
        end_mileage: row.end_mileage,
        total_distance: row.total_distance,
        fuel_consumed: row.fuel_consumed,
        fuel_cost: row.fuel_cost,
        pre_inspection_notes: row.pre_inspection_notes,
        post_inspection_notes: row.post_inspection_notes,
        pre_inspection_photos: None,
        post_inspection_photos: None,
        route_optimization_score: row.route_optimization_score,
        estimated_duration_minutes: row.estimated_duration_minutes,
        actual_duration_minutes: row.actual_duration_minutes,
        route_coordinates: None,
        traffic_conditions: None,
        weather_conditions: None,
        tournee_origin: row.tournee_origin,
        external_tournee_id: row.external_tournee_id,
        integration_id: row.integration_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        deleted_at: row.deleted_at,
    };

    Ok(Json(TourneeResponse::from(tournee)))
}

/// Finalizar una tournée
pub async fn end_tournee(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Path(id): Path<Uuid>,
    Json(end_data): Json<EndTourneeRequest>,
) -> AppResult<Json<TourneeResponse>> {
    // Validar datos de entrada
    end_data.validate()
        .map_err(AppError::Validation)?;

    // Calcular total_distance y actual_duration_minutes
    let total_distance = if let Some(start_mileage) = sqlx::query_scalar!(
        "SELECT start_mileage FROM tournees WHERE id = $1",
        id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?
    .flatten()
    {
        Some(end_data.end_mileage - start_mileage)
    } else {
        None
    };

    let row = sqlx::query!(
        r#"
        UPDATE tournees SET
            tournee_status = 'completed',
            end_time = NOW(),
            end_mileage = $2,
            total_distance = $3,
            fuel_consumed = $4,
            fuel_cost = $5,
            post_inspection_notes = $6,
            actual_duration_minutes = EXTRACT(EPOCH FROM (NOW() - start_time)) / 60,
            updated_at = NOW()
        WHERE id = $1 AND company_id = $7 AND tournee_status = 'in_progress'
        RETURNING 
            id, company_id, driver_id, vehicle_id, tournee_date, tournee_number,
            start_location, end_location, tournee_status as "tournee_status: String",
            start_time, end_time, start_mileage, end_mileage, total_distance,
            fuel_consumed, fuel_cost, pre_inspection_notes, post_inspection_notes,
            route_optimization_score, estimated_duration_minutes, actual_duration_minutes,
            tournee_origin, external_tournee_id, integration_id,
            created_at, updated_at, deleted_at
        "#,
        id,
        end_data.end_mileage,
        total_distance,
        end_data.fuel_consumed,
        end_data.fuel_cost,
        end_data.post_inspection_notes,
        user.company_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound("Tournée no encontrada o no está en progreso".to_string()),
        _ => AppError::Database(e),
    })?;

    let tournee = Tournee {
        id: row.id,
        company_id: row.company_id,
        driver_id: row.driver_id,
        vehicle_id: row.vehicle_id,
        tournee_date: row.tournee_date,
        tournee_number: row.tournee_number,
        start_location: row.start_location,
        end_location: row.end_location,
        tournee_status: TourneeStatus::Completed,
        start_time: row.start_time,
        end_time: row.end_time,
        start_mileage: row.start_mileage,
        end_mileage: row.end_mileage,
        total_distance: row.total_distance,
        fuel_consumed: row.fuel_consumed,
        fuel_cost: row.fuel_cost,
        pre_inspection_notes: row.pre_inspection_notes,
        post_inspection_notes: row.post_inspection_notes,
        pre_inspection_photos: None,
        post_inspection_photos: None,
        route_optimization_score: row.route_optimization_score,
        estimated_duration_minutes: row.estimated_duration_minutes,
        actual_duration_minutes: row.actual_duration_minutes,
        route_coordinates: None,
        traffic_conditions: None,
        weather_conditions: None,
        tournee_origin: row.tournee_origin,
        external_tournee_id: row.external_tournee_id,
        integration_id: row.integration_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        deleted_at: row.deleted_at,
    };

    Ok(Json(TourneeResponse::from(tournee)))
}

/// Eliminar una tournée (soft delete)
pub async fn delete_tournee(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let result = sqlx::query!(
        r#"
        UPDATE tournees 
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
        return Err(AppError::NotFound("Tournée no encontrada".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}