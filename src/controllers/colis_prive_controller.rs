use crate::dto::colis_prive_dto::*;
use crate::repositories::colis_prive_repository::ColisPriveRepository;
use crate::services::colis_prive_service::ColisPriveService;
use crate::services::colis_prive_companies_service;
use crate::services::geocoding_service::GeocodingService;
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
                // Extraer solo la parte del matricule (después del _)
                let matricule_only = if let Some(pos) = auth_data.matricule_chauffeur.rfind('_') {
                    &auth_data.matricule_chauffeur[pos + 1..]
                } else {
                    &auth_data.matricule_chauffeur
                };
                
                log::info!("💾 Guardando token para {}:{}", request.societe, matricule_only);
                
                // Guardar token en cache
                self.repository.save_token(
                    &request.societe,
                    matricule_only,
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
        let mut packages = self.service.get_tournee(
            &token.token,
            &request.matricule,
            &request.societe,
            request.date.as_deref(),
        ).await?;

        let total = packages.len();
        log::info!("✅ Paquetes obtenidos: {}", total);

        // 🗺️ Geocoding automático de paquetes
        log::info!("🗺️ Iniciando geocoding automático de {} paquetes...", packages.len());
        
        // Verificar que tengamos el token de Mapbox
        let mapbox_token = state.config.mapbox_token.clone()
            .ok_or_else(|| AppError::ExternalApi("Mapbox token no configurado".to_string()))?;
        
        let geocoding_service = GeocodingService::new(mapbox_token);

        let mut geocoded_count = 0;
        let mut already_geocoded = 0;

        for package in &mut packages {
            // Si ya tiene coordenadas de Colis Privé, usarlas
            if package.coord_x_destinataire.is_some() && package.coord_y_destinataire.is_some() {
                package.latitude = package.coord_y_destinataire;
                package.longitude = package.coord_x_destinataire;
                already_geocoded += 1;
                continue;
            }

            // Construir dirección completa
            let mut address_parts = Vec::new();
            
            if let Some(addr1) = &package.destinataire_adresse1 {
                address_parts.push(addr1.clone());
            }
            if let Some(addr2) = &package.destinataire_adresse2 {
                if !addr2.trim().is_empty() {
                    address_parts.push(addr2.clone());
                }
            }
            if let Some(cp) = &package.destinataire_cp {
                address_parts.push(cp.clone());
            }
            if let Some(ville) = &package.destinataire_ville {
                address_parts.push(ville.clone());
            }

            let full_address = address_parts.join(", ");
            
            if full_address.is_empty() {
                log::warn!("⚠️ Paquete {} sin dirección válida", package.reference_colis);
                continue;
            }

            // Hacer geocoding
            match geocoding_service.geocode_address(&full_address).await {
                Ok(geo_result) if geo_result.success => {
                    package.latitude = geo_result.latitude;
                    package.longitude = geo_result.longitude;
                    package.formatted_address = geo_result.formatted_address;
                    package.validation_method = Some("geocoded".to_string());
                    package.validation_confidence = Some(0.9); // Alta confianza para Mapbox
                    geocoded_count += 1;
                }
                Ok(_) => {
                    log::warn!("⚠️ No se pudo geocodificar: {}", full_address);
                }
                Err(e) => {
                    log::error!("❌ Error geocodificando {}: {}", full_address, e);
                }
            }
        }

        log::info!("✅ Geocoding completado: {} nuevos, {} ya existentes, {} total", 
            geocoded_count, already_geocoded, packages.len());

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
