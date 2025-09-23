//! Services module
//! 
//! Este módulo contiene la lógica de negocio y servicios de la aplicación.
//! Los servicios encapsulan operaciones complejas que pueden involucrar 
//! múltiples modelos o integraciones externas.

pub mod colis_prive_service;
pub mod colis_prive_companies_service;
// app_version_service eliminado - era para reverse engineering de API móvil
// pub mod colis_prive_flow_service; // Comentado temporalmente por errores de compilación
// pub mod colis_prive_complete_flow_service; // Comentado temporalmente por errores de compilación
pub mod colis_prive_web_service;
pub mod geocoding_service;
pub mod address_validation;
pub mod hybrid_processor;

pub use colis_prive_service::*;
// app_version_service eliminado - era para reverse engineering de API móvil
// pub use colis_prive_flow_service::*; // Comentado temporalmente
// pub use colis_prive_complete_flow_service::*; // Comentado temporalmente
// colis_prive_web_service - no se usa actualmente
pub use geocoding_service::*;
pub use address_validation::*;
// hybrid_processor - no se usa actualmente