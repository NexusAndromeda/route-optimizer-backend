//! Clients - HTTP Clients for External APIs
//! 
//! This module contains HTTP clients for communicating with external APIs.

pub mod colis_prive_client;

// Re-export main types for convenience
pub use colis_prive_client::{
    ColisPriveWebClient,
    ColisDetailResponse,
    ColisDetailData,
    Coordonnees,
    DonneesPhysiques,
    Dimensions,
    HistoriqueItem,
    HorairesLivraison,
    Contact,
    DetailCacheConfig,
};


