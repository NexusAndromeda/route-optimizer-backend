use crate::utils::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, sqlx::FromRow)]
pub struct Address {
    pub id: Uuid,
    pub route_id: Uuid,
    pub address: String,
    pub postal_code: Option<String>,
    pub coordinates: Option<sqlx::types::Json<serde_json::Value>>,
    pub door_codes: Option<String>,
    pub mailbox_access: Option<bool>,
    pub access_instructions: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
}

pub struct AddressRepository {
    pool: PgPool,
}

impl AddressRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        route_id: Uuid,
        address: String,
        postal_code: Option<String>,
        door_codes: Option<String>,
        mailbox_access: Option<bool>,
        access_instructions: Option<String>,
        latitude: Option<f64>,
        longitude: Option<f64>,
    ) -> Result<Address, AppError> {
        let id = Uuid::new_v4();
        
        // Crear punto geométrico si hay coordenadas
        let coordinates_query = if let (Some(lat), Some(lng)) = (latitude, longitude) {
            format!("ST_SetSRID(ST_MakePoint({}, {}), 4326)", lng, lat)
        } else {
            "NULL".to_string()
        };

        let query = format!(
            r#"
            INSERT INTO addresses (id, route_id, address, postal_code, coordinates, door_codes, mailbox_access, access_instructions, created_at)
            VALUES ($1, $2, $3, $4, {}, $5, $6, $7, $8)
            RETURNING id, route_id, address, postal_code, NULL as coordinates, door_codes, mailbox_access, access_instructions, created_at
            "#,
            coordinates_query
        );

        let addr = sqlx::query_as::<_, Address>(&query)
            .bind(id)
            .bind(route_id)
            .bind(address)
            .bind(postal_code)
            .bind(door_codes)
            .bind(mailbox_access.unwrap_or(false))
            .bind(access_instructions)
            .bind(Utc::now())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Error creating address: {}", e)))?;

        Ok(addr)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Address>, AppError> {
        let addr = sqlx::query_as::<_, Address>(
            "SELECT id, route_id, address, postal_code, NULL as coordinates, door_codes, mailbox_access, access_instructions, created_at FROM addresses WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Error finding address: {}", e)))?;

        Ok(addr)
    }

    pub async fn find_by_route(&self, route_id: Uuid) -> Result<Vec<Address>, AppError> {
        let addresses = sqlx::query_as::<_, Address>(
            "SELECT id, route_id, address, postal_code, NULL as coordinates, door_codes, mailbox_access, access_instructions, created_at FROM addresses WHERE route_id = $1 ORDER BY created_at DESC"
        )
        .bind(route_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Error listing addresses: {}", e)))?;

        Ok(addresses)
    }

    pub async fn search_by_address(&self, address_pattern: &str) -> Result<Vec<Address>, AppError> {
        let addresses = sqlx::query_as::<_, Address>(
            "SELECT id, route_id, address, postal_code, NULL as coordinates, door_codes, mailbox_access, access_instructions, created_at FROM addresses WHERE address ILIKE $1 ORDER BY created_at DESC LIMIT 50"
        )
        .bind(format!("%{}%", address_pattern))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Error searching addresses: {}", e)))?;

        Ok(addresses)
    }

    pub async fn update(
        &self,
        id: Uuid,
        door_codes: Option<String>,
        mailbox_access: Option<bool>,
        access_instructions: Option<String>,
    ) -> Result<Address, AppError> {
        let current = self.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("Address not found".to_string()))?;

        let addr = sqlx::query_as::<_, Address>(
            r#"
            UPDATE addresses
            SET door_codes = $2, mailbox_access = $3, access_instructions = $4
            WHERE id = $1
            RETURNING id, route_id, address, postal_code, NULL as coordinates, door_codes, mailbox_access, access_instructions, created_at
            "#
        )
        .bind(id)
        .bind(door_codes.or(current.door_codes))
        .bind(mailbox_access.unwrap_or(current.mailbox_access.unwrap_or(false)))
        .bind(access_instructions.or(current.access_instructions))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Error updating address: {}", e)))?;

        Ok(addr)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM addresses WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Error deleting address: {}", e)))?;

        Ok(())
    }
}
