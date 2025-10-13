use crate::dto::company_dto::{RegisterCompanyRequest, CompanyResponse, ApiResponse};
use crate::dto::auth_dto::{LoginRequest, LoginResponse};
use crate::models::company::Company;
use crate::repositories::company_repository::CompanyRepository;
use crate::utils::errors::AppError;
use crate::utils::jwt::create_jwt_token;
use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::PgPool;

pub struct CompanyController {
    repository: CompanyRepository,
}

impl CompanyController {
    pub fn new(pool: PgPool) -> Self {
        Self {
            repository: CompanyRepository::new(pool),
        }
    }

    pub async fn register(&self, request: RegisterCompanyRequest) -> Result<ApiResponse<CompanyResponse>, AppError> {
        // Validar campos
        if request.company_name.trim().is_empty() {
            return Err(AppError::ValidationError("El nombre de la empresa es requerido".to_string()));
        }

        if request.company_address.trim().is_empty() {
            return Err(AppError::ValidationError("La dirección es requerida".to_string()));
        }

        if request.admin_full_name.trim().is_empty() {
            return Err(AppError::ValidationError("El nombre del administrador es requerido".to_string()));
        }

        if request.admin_email.trim().is_empty() || !request.admin_email.contains('@') {
            return Err(AppError::ValidationError("Email inválido".to_string()));
        }

        if request.admin_password.len() < 8 {
            return Err(AppError::ValidationError("La contraseña debe tener al menos 8 caracteres".to_string()));
        }

        // Validar SIRET si existe
        if let Some(ref siret) = request.company_siret {
            if !siret.is_empty() && (siret.len() != 14 || !siret.chars().all(char::is_numeric)) {
                return Err(AppError::ValidationError("El SIRET debe tener 14 dígitos".to_string()));
            }
        }

        // Verificar que el email no exista
        if self.repository.email_exists(&request.admin_email).await? {
            return Err(AppError::Conflict("El email ya está registrado".to_string()));
        }

        // Verificar que el SIRET no exista
        if let Some(ref siret) = request.company_siret {
            if !siret.is_empty() && self.repository.siret_exists(siret).await? {
                return Err(AppError::Conflict("El SIRET ya está registrado".to_string()));
            }
        }

        // Hash de la contraseña
        let password_hash = hash(&request.admin_password, DEFAULT_COST)
            .map_err(|e| AppError::Internal(format!("Error hashing password: {}", e)))?;

        // Crear empresa
        let company = Company::new(
            request.company_name,
            request.company_address,
            request.company_siret.filter(|s| !s.is_empty()),
            request.admin_full_name,
            request.admin_email,
            password_hash,
        );

        // Guardar en DB
        let saved_company = self.repository.create(&company).await?;

        // Convertir a DTO
        let response = CompanyResponse {
            id: saved_company.id,
            name: saved_company.name,
            address: saved_company.address,
            siret: saved_company.siret,
            admin_full_name: saved_company.admin_full_name,
            admin_email: saved_company.admin_email,
            subscription_plan: saved_company.subscription_plan,
            subscription_status: saved_company.subscription_status,
            created_at: saved_company.created_at,
        };

        Ok(ApiResponse::success_with_message(
            response,
            "Empresa registrada exitosamente".to_string()
        ))
    }

    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse, AppError> {
        // Buscar empresa por email
        let company = self.repository
            .find_by_email(&request.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Credenciales inválidas".to_string()))?;

        // Verificar contraseña
        let valid = verify(&request.password, &company.admin_password_hash)
            .map_err(|e| AppError::Internal(format!("Error verifying password: {}", e)))?;

        if !valid {
            return Err(AppError::Unauthorized("Credenciales inválidas".to_string()));
        }

        // Generar JWT token
        let token = create_jwt_token(&company.id.to_string(), &company.admin_email)
            .map_err(|e| AppError::Internal(format!("Error creating token: {}", e)))?;

        Ok(LoginResponse::success(
            token,
            company.id.to_string(),
            company.name,
        ))
    }

    pub async fn get_by_id(&self, id: uuid::Uuid) -> Result<CompanyResponse, AppError> {
        let company = self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Empresa no encontrada".to_string()))?;

        Ok(CompanyResponse {
            id: company.id,
            name: company.name,
            address: company.address,
            siret: company.siret,
            admin_full_name: company.admin_full_name,
            admin_email: company.admin_email,
            subscription_plan: company.subscription_plan,
            subscription_status: company.subscription_status,
            created_at: company.created_at,
        })
    }
}

