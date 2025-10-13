use crate::models::company::Company;
use crate::utils::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;

pub struct CompanyRepository {
    pool: PgPool,
}

impl CompanyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, company: &Company) -> Result<Company, AppError> {
        let result = sqlx::query_as::<_, Company>(
            r#"
            INSERT INTO companies (
                id, name, address, siret, admin_full_name, admin_email, 
                admin_password_hash, subscription_plan, subscription_status, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#
        )
        .bind(&company.id)
        .bind(&company.name)
        .bind(&company.address)
        .bind(&company.siret)
        .bind(&company.admin_full_name)
        .bind(&company.admin_email)
        .bind(&company.admin_password_hash)
        .bind(&company.subscription_plan)
        .bind(&company.subscription_status)
        .bind(&company.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Error creating company: {}", e)))?;

        Ok(result)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Company>, AppError> {
        let result = sqlx::query_as::<_, Company>(
            "SELECT * FROM companies WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Error finding company: {}", e)))?;

        Ok(result)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<Company>, AppError> {
        let result = sqlx::query_as::<_, Company>(
            "SELECT * FROM companies WHERE admin_email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Error finding company by email: {}", e)))?;

        Ok(result)
    }

    pub async fn email_exists(&self, email: &str) -> Result<bool, AppError> {
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM companies WHERE admin_email = $1)"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Error checking email: {}", e)))?;

        Ok(result.0)
    }

    pub async fn siret_exists(&self, siret: &str) -> Result<bool, AppError> {
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM companies WHERE siret = $1)"
        )
        .bind(siret)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Error checking siret: {}", e)))?;

        Ok(result.0)
    }

    pub async fn update(&self, company: &Company) -> Result<Company, AppError> {
        let result = sqlx::query_as::<_, Company>(
            r#"
            UPDATE companies
            SET name = $2, address = $3, siret = $4, admin_full_name = $5,
                subscription_plan = $6, subscription_status = $7
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(&company.id)
        .bind(&company.name)
        .bind(&company.address)
        .bind(&company.siret)
        .bind(&company.admin_full_name)
        .bind(&company.subscription_plan)
        .bind(&company.subscription_status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Error updating company: {}", e)))?;

        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM companies WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Error deleting company: {}", e)))?;

        Ok(())
    }

    pub async fn list_all(&self) -> Result<Vec<Company>, AppError> {
        let result = sqlx::query_as::<_, Company>(
            "SELECT * FROM companies ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Error listing companies: {}", e)))?;

        Ok(result)
    }
}

