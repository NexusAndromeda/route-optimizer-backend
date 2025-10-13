//! Cache
//! 
//! Este módulo contiene los sistemas de cache.

pub mod redis_client;
// pub mod detail_cache; // Comentado - legacy, necesita refactoring
pub mod cache_config;

pub use cache_config::CacheConfig;