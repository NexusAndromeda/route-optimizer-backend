//! Handlers de Companies
//! 
//! Este módulo maneja las operaciones CRUD para empresas.

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::company::{Company, CompanyResponse, CreateCompanyRequest},
    utils::errors::{AppError, AppResult},
    middleware::auth::AuthenticatedUser,
};

/// Handler para listar empresas (solo la empresa del usuario autenticado)
pub async fn get_companies(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
) -> AppResult<Json<Vec<CompanyResponse>>> {
    let row = sqlx::query!(
        r#"
        SELECT 
            id, name, address, subscription_plan, subscription_status, 
            max_drivers, max_vehicles, created_at, updated_at, deleted_at
        FROM companies 
        WHERE id = $1 
        AND deleted_at IS NULL
        "#,
        user.company_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let companies = if let Some(row) = row {
        vec![CompanyResponse {
            id: row.id,
            name: row.name,
            address: row.address,
            subscription_plan: row.subscription_plan,
            subscription_status: row.subscription_status,
            max_drivers: row.max_drivers,
            max_vehicles: row.max_vehicles,
            created_at: row.created_at.unwrap_or_else(|| chrono::Utc::now()),
            updated_at: row.updated_at.unwrap_or_else(|| chrono::Utc::now()),
        }]
    } else {
        vec![]
    };

    Ok(Json(companies))
}

/// Handler para crear empresa (solo admins)
pub async fn create_company(
    Extension(_user): Extension<AuthenticatedUser>,
    State(_state): State<crate::state::AppState>,
    Json(_company_data): Json<CreateCompanyRequest>,
) -> AppResult<Json<CompanyResponse>> {
    // Por ahora retorna Not Implemented
    Err(AppError::NotImplemented("Create company no implementado aún".to_string()))
}

/// Handler para obtener empresa por ID
pub async fn get_company(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<crate::state::AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<CompanyResponse>> {
    // Solo puede ver su propia empresa
    if id != user.company_id {
        return Err(AppError::Forbidden("No tienes acceso a esta empresa".to_string()));
    }

    let row = sqlx::query!(
        r#"
        SELECT 
            id, name, address, subscription_plan, subscription_status, 
            max_drivers, max_vehicles, created_at, updated_at, deleted_at
        FROM companies 
        WHERE id = $1 
        AND deleted_at IS NULL
        "#,
        id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?
    .ok_or_else(|| AppError::NotFound("Empresa no encontrada".to_string()))?;

    let company_response = CompanyResponse {
        id: row.id,
        name: row.name,
        address: row.address,
        subscription_plan: row.subscription_plan,
        subscription_status: row.subscription_status,
        max_drivers: row.max_drivers,
        max_vehicles: row.max_vehicles,
        created_at: row.created_at.unwrap_or_else(|| chrono::Utc::now()),
        updated_at: row.updated_at.unwrap_or_else(|| chrono::Utc::now()),
    };

    Ok(Json(company_response))
}

/// Handler para actualizar empresa (versión simplificada)
pub async fn update_company(
    Extension(_user): Extension<AuthenticatedUser>,
    State(_state): State<crate::state::AppState>,
    Path(_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // Por ahora retorna Not Implemented
    Err(AppError::NotImplemented("Update company no implementado aún".to_string()))
}

/// Handler para eliminar empresa (soft delete)
pub async fn delete_company(
    Extension(_user): Extension<AuthenticatedUser>,
    State(_state): State<crate::state::AppState>,
    Path(_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // Por ahora retorna Not Implemented - eliminar empresa es operación muy delicada
    Err(AppError::NotImplemented("Delete company no implementado aún".to_string()))
}
