//! Handlers de Analytics
//! 
//! Este módulo maneja las operaciones para métricas y analytics.

use axum::{
    extract::{Query, State},
    Json,
};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    models::analytics::{
        PerformanceAnalytics, DashboardSummary, AnalyticsResponse,
        CreateAnalyticsRequest, AnalyticsFilters,
    },
    utils::errors::{AppError, AppResult},
    middleware::auth::AuthenticatedUser,
};

/// Obtener resumen del dashboard
pub async fn get_dashboard_summary(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
) -> AppResult<Json<DashboardSummary>> {
    let today = chrono::Utc::now().date_naive();

    // Obtener resumen de tournées
    let tournees_summary = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_tournees,
            COUNT(CASE WHEN tournee_status = 'completed' THEN 1 END) as completed_tournees,
            COUNT(CASE WHEN tournee_status = 'in_progress' THEN 1 END) as active_tournees
        FROM tournees 
        WHERE company_id = $1 
        AND tournee_date = $2
        AND deleted_at IS NULL
        "#,
        user.company_id,
        today
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    // Obtener resumen de paquetes
    let packages_summary = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_packages,
            COUNT(CASE WHEN delivery_status = 'delivered' THEN 1 END) as delivered_packages,
            COUNT(CASE WHEN delivery_status = 'failed' THEN 1 END) as failed_packages
        FROM packages p
        JOIN tournees t ON p.tournee_id = t.id
        WHERE p.company_id = $1 
        AND t.tournee_date = $2
        AND p.deleted_at IS NULL
        "#,
        user.company_id,
        today
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    // Obtener métricas de rendimiento
    let performance_metrics = sqlx::query!(
        r#"
        SELECT 
            AVG(actual_duration_minutes) as avg_delivery_time,
            AVG(route_optimization_score) as avg_route_efficiency,
            SUM(total_distance) as total_distance
        FROM tournees 
        WHERE company_id = $1 
        AND tournee_date = $2
        AND tournee_status = 'completed'
        AND deleted_at IS NULL
        "#,
        user.company_id,
        today
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let summary = DashboardSummary {
        company_id: user.company_id,
        date: today,
        total_tournees: tournees_summary.total_tournees.unwrap_or(0) as i32,
        completed_tournees: tournees_summary.completed_tournees.unwrap_or(0) as i32,
        active_tournees: tournees_summary.active_tournees.unwrap_or(0) as i32,
        total_packages: packages_summary.total_packages.unwrap_or(0) as i32,
        delivered_packages: packages_summary.delivered_packages.unwrap_or(0) as i32,
        failed_packages: packages_summary.failed_packages.unwrap_or(0) as i32,
        average_delivery_time_minutes: performance_metrics.avg_delivery_time
            .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0),
        average_route_efficiency: performance_metrics.avg_route_efficiency
            .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0),
        total_distance_km: performance_metrics.total_distance
            .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0),
        total_revenue: None,
        total_costs: None,
        profit_margin: None,
        average_customer_rating: None,
        total_complaints: 0,
    };

    Ok(Json(summary))
}

/// Obtener métricas de rendimiento por tournée
pub async fn get_performance_by_tournee(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Query(filters): Query<AnalyticsFilters>,
) -> AppResult<Json<Vec<AnalyticsResponse>>> {
    let date_from = filters.date_from.unwrap_or_else(|| {
        chrono::Utc::now().date_naive() - chrono::Duration::days(30)
    });
    let date_to = filters.date_to.unwrap_or_else(|| chrono::Utc::now().date_naive());

    let rows = sqlx::query!(
        r#"
        SELECT 
            t.id,
            t.company_id,
            t.id as tournee_id,
            t.driver_id,
            t.vehicle_id,
            COALESCE(t.actual_duration_minutes, 0) as total_time_minutes,
            COALESCE(t.actual_duration_minutes, 0) as driving_time_minutes,
            0 as waiting_time_minutes,
            COALESCE(t.total_distance, 0) as total_distance_km,
            COALESCE(t.route_optimization_score, 0) as route_efficiency,
            COUNT(p.id) FILTER (WHERE p.delivery_status = 'delivered') as packages_delivered,
            COUNT(p.id) FILTER (WHERE p.delivery_status = 'failed') as packages_failed,
            CASE 
                WHEN COUNT(p.id) > 0 THEN 
                    (COUNT(p.id) FILTER (WHERE p.delivery_status = 'delivered')::float / COUNT(p.id)::float) * 100
                ELSE 0 
            END as delivery_success_rate,
            COALESCE(t.fuel_consumed, 0) as fuel_consumed_liters,
            CASE 
                WHEN t.fuel_consumed > 0 AND t.total_distance > 0 THEN 
                    t.total_distance / t.fuel_consumed
                ELSE 0 
            END as fuel_efficiency_km_l,
            COALESCE(t.fuel_cost, 0) as total_cost,
            CASE 
                WHEN COUNT(p.id) > 0 THEN 
                    COALESCE(t.fuel_cost, 0) / COUNT(p.id)
                ELSE 0 
            END as cost_per_package,
            CASE 
                WHEN t.total_distance > 0 THEN 
                    COALESCE(t.fuel_cost, 0) / t.total_distance
                ELSE 0 
            END as cost_per_km,
            NULL::float as customer_rating,
            0 as complaints_count,
            t.tournee_date as date,
            t.created_at,
            t.updated_at
        FROM tournees t
        LEFT JOIN packages p ON t.id = p.tournee_id AND p.deleted_at IS NULL
        WHERE t.company_id = $1
        AND t.tournee_date BETWEEN $2 AND $3
        AND t.tournee_status = 'completed'
        AND t.deleted_at IS NULL
        GROUP BY t.id, t.company_id, t.driver_id, t.vehicle_id, t.tournee_date,
                 t.actual_duration_minutes, t.total_distance, t.route_optimization_score,
                 t.fuel_consumed, t.fuel_cost, t.created_at, t.updated_at
        ORDER BY t.tournee_date DESC, t.created_at DESC
        "#,
        user.company_id,
        date_from,
        date_to
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let analytics: Vec<AnalyticsResponse> = rows
        .into_iter()
        .map(|row| AnalyticsResponse {
            id: row.id,
            company_id: row.company_id,
            tournee_id: Some(row.tournee_id),
            driver_id: Some(row.driver_id),
            vehicle_id: Some(row.vehicle_id),
            total_time_minutes: row.total_time_minutes.unwrap_or(0) as i32,
            driving_time_minutes: row.driving_time_minutes.unwrap_or(0) as i32,
            waiting_time_minutes: row.waiting_time_minutes.expect("waiting_time_minutes should not be null"),
            total_distance_km: row.total_distance_km
                .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0),
            route_efficiency: row.route_efficiency
                .map(|e| e.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0),
            packages_delivered: row.packages_delivered.unwrap_or(0) as i32,
            packages_failed: row.packages_failed.unwrap_or(0) as i32,
            delivery_success_rate: row.delivery_success_rate.unwrap_or(0.0),
            fuel_consumed_liters: row.fuel_consumed_liters
                .map(|f| f.to_string().parse::<f64>().unwrap_or(0.0)),
            fuel_efficiency_km_l: row.fuel_efficiency_km_l.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            total_cost: row.total_cost,
            cost_per_package: row.cost_per_package,
            cost_per_km: row.cost_per_km,
            customer_rating: row.customer_rating,
            complaints_count: row.complaints_count.expect("complaints_count should not be null"),
            date: row.date,
            created_at: row.created_at.unwrap_or_else(|| chrono::Utc::now()),
            updated_at: row.updated_at.unwrap_or_else(|| chrono::Utc::now()),
        })
        .collect();

    Ok(Json(analytics))
}

/// Obtener métricas agregadas por conductor
pub async fn get_driver_performance(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Query(filters): Query<AnalyticsFilters>,
) -> AppResult<Json<Vec<AnalyticsResponse>>> {
    let date_from = filters.date_from.unwrap_or_else(|| {
        chrono::Utc::now().date_naive() - chrono::Duration::days(30)
    });
    let date_to = filters.date_to.unwrap_or_else(|| chrono::Utc::now().date_naive());

    let rows = sqlx::query!(
        r#"
        SELECT 
            t.driver_id as id,
            t.company_id,
            NULL::uuid as tournee_id,
            t.driver_id,
            NULL::uuid as vehicle_id,
            SUM(COALESCE(t.actual_duration_minutes, 0)) as total_time_minutes,
            SUM(COALESCE(t.actual_duration_minutes, 0)) as driving_time_minutes,
            0 as waiting_time_minutes,
            SUM(COALESCE(t.total_distance, 0)) as total_distance_km,
            AVG(COALESCE(t.route_optimization_score, 0)) as route_efficiency,
            COUNT(p.id) FILTER (WHERE p.delivery_status = 'delivered') as packages_delivered,
            COUNT(p.id) FILTER (WHERE p.delivery_status = 'failed') as packages_failed,
            CASE 
                WHEN COUNT(p.id) > 0 THEN 
                    (COUNT(p.id) FILTER (WHERE p.delivery_status = 'delivered')::float / COUNT(p.id)::float) * 100
                ELSE 0 
            END as delivery_success_rate,
            SUM(COALESCE(t.fuel_consumed, 0)) as fuel_consumed_liters,
            CASE 
                WHEN SUM(t.fuel_consumed) > 0 AND SUM(t.total_distance) > 0 THEN 
                    SUM(t.total_distance) / SUM(t.fuel_consumed)
                ELSE 0 
            END as fuel_efficiency_km_l,
            SUM(COALESCE(t.fuel_cost, 0)) as total_cost,
            CASE 
                WHEN COUNT(p.id) > 0 THEN 
                    SUM(COALESCE(t.fuel_cost, 0)) / COUNT(p.id)
                ELSE 0 
            END as cost_per_package,
            CASE 
                WHEN SUM(t.total_distance) > 0 THEN 
                    SUM(COALESCE(t.fuel_cost, 0)) / SUM(t.total_distance)
                ELSE 0 
            END as cost_per_km,
            NULL::float as customer_rating,
            0 as complaints_count,
            $3::date as date,
            MIN(t.created_at) as created_at,
            MAX(t.updated_at) as updated_at
        FROM tournees t
        LEFT JOIN packages p ON t.id = p.tournee_id AND p.deleted_at IS NULL
        WHERE t.company_id = $1
        AND t.tournee_date BETWEEN $2 AND $3
        AND t.tournee_status = 'completed'
        AND t.deleted_at IS NULL
        GROUP BY t.driver_id, t.company_id
        ORDER BY packages_delivered DESC
        "#,
        user.company_id,
        date_from,
        date_to
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let analytics: Vec<AnalyticsResponse> = rows
        .into_iter()
        .map(|row| AnalyticsResponse {
            id: row.id,
            company_id: row.company_id,
            tournee_id: row.tournee_id,
            driver_id: Some(row.driver_id),
            vehicle_id: row.vehicle_id,
            total_time_minutes: row.total_time_minutes.unwrap_or(0) as i32,
            driving_time_minutes: row.driving_time_minutes.unwrap_or(0) as i32,
            waiting_time_minutes: row.waiting_time_minutes.expect("waiting_time_minutes should not be null"),
            total_distance_km: row.total_distance_km
                .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0),
            route_efficiency: row.route_efficiency
                .map(|e| e.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0),
            packages_delivered: row.packages_delivered.unwrap_or(0) as i32,
            packages_failed: row.packages_failed.unwrap_or(0) as i32,
            delivery_success_rate: row.delivery_success_rate.unwrap_or(0.0),
            fuel_consumed_liters: row.fuel_consumed_liters
                .map(|f| f.to_string().parse::<f64>().unwrap_or(0.0)),
            fuel_efficiency_km_l: row.fuel_efficiency_km_l.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            total_cost: row.total_cost,
            cost_per_package: row.cost_per_package,
            cost_per_km: row.cost_per_km,
            customer_rating: row.customer_rating,
            complaints_count: row.complaints_count.expect("complaints_count should not be null"),
            date: row.date.expect("date should not be null"),
            created_at: row.created_at.unwrap_or_else(|| chrono::Utc::now()),
            updated_at: row.updated_at.unwrap_or_else(|| chrono::Utc::now()),
        })
        .collect();

    Ok(Json(analytics))
}

/// Obtener métricas agregadas por vehículo
pub async fn get_vehicle_performance(
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Query(filters): Query<AnalyticsFilters>,
) -> AppResult<Json<Vec<AnalyticsResponse>>> {
    let date_from = filters.date_from.unwrap_or_else(|| {
        chrono::Utc::now().date_naive() - chrono::Duration::days(30)
    });
    let date_to = filters.date_to.unwrap_or_else(|| chrono::Utc::now().date_naive());

    let rows = sqlx::query!(
        r#"
        SELECT 
            t.vehicle_id as id,
            t.company_id,
            NULL::uuid as tournee_id,
            NULL::uuid as driver_id,
            t.vehicle_id,
            SUM(COALESCE(t.actual_duration_minutes, 0)) as total_time_minutes,
            SUM(COALESCE(t.actual_duration_minutes, 0)) as driving_time_minutes,
            0 as waiting_time_minutes,
            SUM(COALESCE(t.total_distance, 0)) as total_distance_km,
            AVG(COALESCE(t.route_optimization_score, 0)) as route_efficiency,
            COUNT(p.id) FILTER (WHERE p.delivery_status = 'delivered') as packages_delivered,
            COUNT(p.id) FILTER (WHERE p.delivery_status = 'failed') as packages_failed,
            CASE 
                WHEN COUNT(p.id) > 0 THEN 
                    (COUNT(p.id) FILTER (WHERE p.delivery_status = 'delivered')::float / COUNT(p.id)::float) * 100
                ELSE 0 
            END as delivery_success_rate,
            SUM(COALESCE(t.fuel_consumed, 0)) as fuel_consumed_liters,
            CASE 
                WHEN SUM(t.fuel_consumed) > 0 AND SUM(t.total_distance) > 0 THEN 
                    SUM(t.total_distance) / SUM(t.fuel_consumed)
                ELSE 0 
            END as fuel_efficiency_km_l,
            SUM(COALESCE(t.fuel_cost, 0)) as total_cost,
            CASE 
                WHEN COUNT(p.id) > 0 THEN 
                    SUM(COALESCE(t.fuel_cost, 0)) / COUNT(p.id)
                ELSE 0 
            END as cost_per_package,
            CASE 
                WHEN SUM(t.total_distance) > 0 THEN 
                    SUM(COALESCE(t.fuel_cost, 0)) / SUM(t.total_distance)
                ELSE 0 
            END as cost_per_km,
            NULL::float as customer_rating,
            0 as complaints_count,
            $3::date as date,
            MIN(t.created_at) as created_at,
            MAX(t.updated_at) as updated_at
        FROM tournees t
        LEFT JOIN packages p ON t.id = p.tournee_id AND p.deleted_at IS NULL
        WHERE t.company_id = $1
        AND t.tournee_date BETWEEN $2 AND $3
        AND t.tournee_status = 'completed'
        AND t.deleted_at IS NULL
        GROUP BY t.vehicle_id, t.company_id
        ORDER BY packages_delivered DESC
        "#,
        user.company_id,
        date_from,
        date_to
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let analytics: Vec<AnalyticsResponse> = rows
        .into_iter()
        .map(|row| AnalyticsResponse {
            id: row.id,
            company_id: row.company_id,
            tournee_id: row.tournee_id,
            driver_id: row.driver_id,
            vehicle_id: Some(row.vehicle_id),
            total_time_minutes: row.total_time_minutes.unwrap_or(0) as i32,
            driving_time_minutes: row.driving_time_minutes.unwrap_or(0) as i32,
            waiting_time_minutes: row.waiting_time_minutes.expect("waiting_time_minutes should not be null"),
            total_distance_km: row.total_distance_km
                .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0),
            route_efficiency: row.route_efficiency
                .map(|e| e.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0),
            packages_delivered: row.packages_delivered.unwrap_or(0) as i32,
            packages_failed: row.packages_failed.unwrap_or(0) as i32,
            delivery_success_rate: row.delivery_success_rate.unwrap_or(0.0),
            fuel_consumed_liters: row.fuel_consumed_liters
                .map(|f| f.to_string().parse::<f64>().unwrap_or(0.0)),
            fuel_efficiency_km_l: row.fuel_efficiency_km_l.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            total_cost: row.total_cost,
            cost_per_package: row.cost_per_package,
            cost_per_km: row.cost_per_km,
            customer_rating: row.customer_rating,
            complaints_count: row.complaints_count.expect("complaints_count should not be null"),
            date: row.date.expect("date should not be null"),
            created_at: row.created_at.unwrap_or_else(|| chrono::Utc::now()),
            updated_at: row.updated_at.unwrap_or_else(|| chrono::Utc::now()),
        })
        .collect();

    Ok(Json(analytics))
}