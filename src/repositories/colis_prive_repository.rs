use crate::state::AuthToken;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// Repository para manejar el cache de tokens SSO de Colis Privé
pub struct ColisPriveRepository {
    auth_tokens: Arc<RwLock<HashMap<String, AuthToken>>>,
}

impl ColisPriveRepository {
    pub fn new(auth_tokens: Arc<RwLock<HashMap<String, AuthToken>>>) -> Self {
        Self { auth_tokens }
    }

    pub async fn get_token(&self, societe: &str, matricule: &str) -> Option<AuthToken> {
        let tokens = self.auth_tokens.read().await;
        let key = format!("{}:{}", societe, matricule);
        tokens.get(&key).cloned()
    }

    pub async fn save_token(&self, societe: &str, matricule: &str, token: AuthToken) {
        let mut tokens = self.auth_tokens.write().await;
        let key = format!("{}:{}", societe, matricule);
        tokens.insert(key, token);
    }

    pub async fn remove_token(&self, societe: &str, matricule: &str) {
        let mut tokens = self.auth_tokens.write().await;
        let key = format!("{}:{}", societe, matricule);
        tokens.remove(&key);
    }

    pub async fn token_exists(&self, societe: &str, matricule: &str) -> bool {
        let tokens = self.auth_tokens.read().await;
        let key = format!("{}:{}", societe, matricule);
        tokens.contains_key(&key)
    }
}

