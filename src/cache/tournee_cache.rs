//! Cache de tournée mínima
//! 
//! Este módulo contiene la funcionalidad mínima de cache para tournées.

use anyhow::Result;

/// Cache mínima para tournées
#[derive(Clone)]
pub struct TourneeCache {
    pub enabled: bool,
}

impl TourneeCache {
    pub fn new() -> Self {
        Self { enabled: false }
    }
    
    pub async fn get(&self, _key: &str) -> Result<Option<String>> {
        Ok(None)
    }
    
    pub async fn set(&self, _key: &str, _value: &str, _ttl: u64) -> Result<()> {
        Ok(())
    }
}
