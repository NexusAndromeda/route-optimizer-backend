use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Company {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub siret: Option<String>,
    pub admin_full_name: String,
    pub admin_email: String,
    #[serde(skip_serializing)]
    pub admin_password_hash: String,
    pub subscription_plan: String,
    pub subscription_status: String,
    pub created_at: DateTime<Utc>,
}

impl Company {
    pub fn new(
        name: String,
        address: String,
        siret: Option<String>,
        admin_full_name: String,
        admin_email: String,
        admin_password_hash: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            address,
            siret,
            admin_full_name,
            admin_email,
            admin_password_hash,
            subscription_plan: "basic".to_string(),
            subscription_status: "active".to_string(),
            created_at: Utc::now(),
        }
    }
}
