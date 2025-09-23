use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColisPriveCompany {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColisPriveCompanyListResponse {
    pub success: bool,
    pub companies: Vec<ColisPriveCompany>,
    pub message: Option<String>,
}

impl Default for ColisPriveCompanyListResponse {
    fn default() -> Self {
        Self {
            success: true,
            companies: vec![
                ColisPriveCompany {
                    code: "02".to_string(),
                    name: "Colis Privé 02".to_string(),
                    description: Some("Empresa principal 02".to_string()),
                },
                ColisPriveCompany {
                    code: "04".to_string(), 
                    name: "Colis Privé 04".to_string(),
                    description: Some("Empresa principal 04".to_string()),
                },
                ColisPriveCompany {
                    code: "16".to_string(),
                    name: "Colis Privé 16".to_string(), 
                    description: Some("Empresa principal 16".to_string()),
                },
                ColisPriveCompany {
                    code: "37".to_string(),
                    name: "Colis Privé 37".to_string(),
                    description: Some("Empresa principal 37".to_string()),
                },
                ColisPriveCompany {
                    code: "CP".to_string(),
                    name: "Colis Privé CP".to_string(),
                    description: Some("Empresa principal CP".to_string()),
                },
            ],
            message: Some("Empresas disponibles cargadas exitosamente".to_string()),
        }
    }
}
