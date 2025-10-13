use crate::dto::colis_prive_dto::*;
use crate::repositories::colis_prive_repository::ColisPriveRepository;
use crate::services::colis_prive_service::ColisPriveService;
use crate::services::colis_prive_companies_service;
use crate::utils::errors::AppError;
use crate::state::AppState;

pub struct ColisPriveController {
    repository: ColisPriveRepository,
    service: ColisPriveService,
}

impl ColisPriveController {
    pub fn new(state: &AppState) -> Self {
        Self {
            repository: ColisPriveRepository::new(state.auth_tokens.clone()),
            service: ColisPriveService::new(state.http_client.clone(), state.config.clone()),
        }
    }

    pub async fn authenticate(
        &self,
        request: ColisPriveAuthRequest,
    ) -> Result<ColisPriveAuthResponse, AppError> {
        log::info!("🔐 Autenticando usuario: {}", request.username);

        // Llamar al servicio para autenticar
        match self.service.authenticate(&request.username, &request.password, &request.societe).await {
            Ok(auth_data) => {
                // Guardar token en cache
                self.repository.save_token(
                    &request.societe,
                    &auth_data.matricule_chauffeur,
                    crate::state::AuthToken::new(
                        auth_data.sso_token.clone(),
                        request.username.clone(),
                        request.societe.clone(),
                        24, // expires in 24 hours
                    )
                ).await;

                log::info!("✅ Autenticación exitosa para: {}", request.username);

                Ok(ColisPriveAuthResponse {
                    success: true,
                    message: Some("Autenticación exitosa".to_string()),
                    authentication: Some(ColisPriveAuthData {
                        sso_token: auth_data.sso_token,
                        matricule_chauffeur: auth_data.matricule_chauffeur,
                        nom_chauffeur: auth_data.nom_chauffeur,
                        societe: request.societe,
                        expires_at: auth_data.expires_at,
                    }),
                    error: None,
                })
            }
            Err(e) => {
                log::error!("❌ Error en autenticación: {}", e);
                Ok(ColisPriveAuthResponse {
                    success: false,
                    message: None,
                    authentication: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    pub async fn get_packages(
        &self,
        request: GetPackagesRequest,
        state: &AppState,
    ) -> Result<PackagesResponse, AppError> {
        log::info!("📦 Obteniendo paquetes para: {}:{}", request.societe, request.matricule);

        // Obtener token del cache
        let token = self.repository
            .get_token(&request.societe, &request.matricule)
            .await
            .ok_or_else(|| AppError::Unauthorized("Token no encontrado. Por favor, autentíquese primero.".to_string()))?;

        // Verificar si el token expiró
        if token.is_expired() {
            log::warn!("⚠️ Token expirado, removiendo del cache");
            self.repository.remove_token(&request.societe, &request.matricule).await;
            return Err(AppError::Unauthorized("Token expirado. Por favor, autentíquese nuevamente.".to_string()));
        }

        // Llamar al servicio para obtener paquetes
        let packages = self.service.get_tournee(
            &token.token,
            &request.matricule,
            &request.societe,
            request.date.as_deref(),
        ).await?;

        let total = packages.len();
        log::info!("✅ Paquetes obtenidos: {}", total);

        Ok(PackagesResponse {
            success: true,
            packages,
            total,
        })
    }

    pub async fn optimize_route(
        &self,
        request: OptimizeRouteRequest,
        state: &AppState,
    ) -> Result<OptimizeRouteResponse, AppError> {
        log::info!("🔄 Optimizando ruta para: {}:{}", request.societe, request.matricule);

        // Obtener token del cache
        let token = self.repository
            .get_token(&request.societe, &request.matricule)
            .await
            .ok_or_else(|| AppError::Unauthorized("Token no encontrado. Por favor, autentíquese primero.".to_string()))?;

        // Verificar si el token expiró
        if token.is_expired() {
            log::warn!("⚠️ Token expirado");
            self.repository.remove_token(&request.societe, &request.matricule).await;
            return Err(AppError::Unauthorized("Token expirado. Por favor, autentíquese nuevamente.".to_string()));
        }

        // Llamar al servicio para optimizar
        let optimized_data = self.service.optimize_tournee(
            &token.token,
            &request.matricule,
            &request.societe,
        ).await?;

        log::info!("✅ Ruta optimizada");

        Ok(OptimizeRouteResponse {
            success: true,
            message: Some("Ruta optimizada exitosamente".to_string()),
            data: Some(OptimizationData {
                matricule_chauffeur: optimized_data.matricule_chauffeur,
                date_tournee: optimized_data.date_tournee,
                optimized_packages: optimized_data.packages,
            }),
        })
    }

    pub async fn get_companies() -> Result<CompaniesListResponse, AppError> {
        log::info!("🏢 Obteniendo lista de empresas");

        let companies = colis_prive_companies_service::fetch_all_companies().await?;

        log::info!("✅ Empresas obtenidas: {}", companies.len());

        let company_list: Vec<CompanyInfo> = companies
            .into_iter()
            .map(|c| CompanyInfo {
                code: c.code,
                name: c.name,
                description: c.description,
            })
            .collect();

        Ok(CompaniesListResponse {
            success: true,
            companies: company_list,
        })
    }
}
