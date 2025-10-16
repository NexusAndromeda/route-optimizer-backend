use crate::services::auth_service::AuthService;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

/// Inicializa el AuthService de manera async
pub async fn init_auth_service() -> Result<Arc<Mutex<AuthService>>> {
    let auth_service = AuthService::new().await?;
    Ok(Arc::new(Mutex::new(auth_service)))
}
