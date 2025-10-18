//! Servicios
//! 
//! Este módulo contiene los servicios de la aplicación.

pub mod colis_prive_service;
pub mod colis_prive_companies_service;
pub mod geocoding_service;
pub mod address_validation;
pub mod address_matching_service;
pub mod package_processing_service;
pub mod address_cache_service;
// pub mod mapbox_optimization_service; // Deshabilitado hasta tener acceso a Mapbox v2 Beta
// pub mod hybrid_processor; // Comentado - legacy, necesita refactoring