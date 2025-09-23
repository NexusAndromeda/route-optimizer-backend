use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;
use crate::services::geocoding_service::{GeocodingService, GeocodingResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidatedAddress {
    pub success: bool,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub formatted_address: Option<String>,
    pub original_address: String,
    pub validation_method: ValidationMethod,
    pub confidence: ValidationConfidence,
    pub warnings: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ValidationMethod {
    Original,
    Cleaned,
    CompletedWithSector,
    PartialSearch,
    ManualRequired,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ValidationConfidence {
    High,    // Dirección original válida
    Medium,  // Dirección limpiada o completada
    Low,     // Búsqueda parcial
    None,    // Requiere intervención manual
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressValidationResult {
    pub total_addresses: usize,
    pub auto_validated: usize,
    pub cleaned_auto: usize,
    pub completed_auto: usize,
    pub partial_found: usize,
    pub requires_manual: usize,
    pub validated_addresses: Vec<ValidatedAddress>,
    pub warnings: Vec<String>,
}

pub struct AddressValidator {
    geocoding_service: GeocodingService,
    client_names: Vec<String>,
    sector_mapping: HashMap<String, String>,
    street_regex: Regex,
    number_regex: Regex,
    duplicate_number_regex: Regex,
    district_in_middle_regex: Regex,
    incomplete_address_regex: Regex,
    separated_numbers_regex: Regex,
}

impl AddressValidator {
    pub fn new(geocoding_service: GeocodingService) -> Self {
        let client_names = vec![
            "MARTIN".to_string(), "DUBOIS".to_string(), "DURAND".to_string(), "MOREAU".to_string(), "LAURENT".to_string(), "BERNARD".to_string(), "THOMAS".to_string(), "PETIT".to_string(),
            "ROBERT".to_string(), "RICHARD".to_string(), "DURAND".to_string(), "DUBOIS".to_string(), "MOREAU".to_string(), "LAURENT".to_string(), "SIMON".to_string(), "MICHEL".to_string(),
            "LEFEBVRE".to_string(), "LEROY".to_string(), "ROUX".to_string(), "DAVID".to_string(), "BERTRAND".to_string(), "MOREL".to_string(), "FOURNIER".to_string(), "GIRARD".to_string(),
            "BONNET".to_string(), "DUPONT".to_string(), "LAMBERT".to_string(), "FONTAINE".to_string(), "ROUSSEAU".to_string(), "VINCENT".to_string(), "MULLER".to_string(), "LEFEVRE".to_string(),
            "ANDRE".to_string(), "LEROY".to_string(), "MARTINEZ".to_string(), "LEGALL".to_string(), "GARCIA".to_string(), "DAVID".to_string(), "BERNARD".to_string(), "THOMAS".to_string()
        ];

        let mut sector_mapping = HashMap::new();
        // Mapeo de sectores basado en el username
        sector_mapping.insert("TEST_USER".to_string(), "CE18".to_string());
        sector_mapping.insert("CE18".to_string(), "CE18".to_string());
        // Agregar más mapeos según sea necesario

        // 🆕 REGEX para reconocer calles francesas
        let street_regex = Regex::new(r"(?i)(rue|avenue|boulevard|place|impasse|allée|chemin|route|passage|square|quai|esplanade|cours|villa|résidence|lotissement|zone|parc|cité|hameau|lieu-dit)\s+([^,]+)").unwrap();
        
        // 🆕 REGEX para detectar números al final
        let number_regex = Regex::new(r"(.+?)\s+(\d+)\s*$").unwrap();
        
        // 🆕 REGEX para detectar números duplicados (ej: "35 35 RUE")
        // Nota: Rust no soporta backreferences, usamos una aproximación diferente
        let duplicate_number_regex = Regex::new(r"(\d+)\s+\d+\s+").unwrap();
        
        // 🆕 REGEX para detectar distrito en medio (ej: "18EME ARRONDISSEMENT")
        let district_in_middle_regex = Regex::new(r"(?i)(\d+EME\s+ARRONDISSEMENT)").unwrap();
        
        // 🆕 REGEX para detectar direcciones incompletas (solo número y código postal)
        let incomplete_address_regex = Regex::new(r"^(\d+),\s*(\d{5})\s+PARIS$").unwrap();
        
        // 🆕 REGEX para detectar números separados (ej: "6 7 IMP" -> tomar el último)
        let separated_numbers_regex = Regex::new(r"(\d+)\s+(\d+)\s+").unwrap();

        Self {
            geocoding_service,
            client_names,
            sector_mapping,
            street_regex,
            number_regex,
            duplicate_number_regex,
            district_in_middle_regex,
            incomplete_address_regex,
            separated_numbers_regex,
        }
    }

    /// Validación inteligente de una dirección con múltiples intentos
    pub async fn validate_address(
        &self,
        address: &str,
        username: &str,
    ) -> Result<ValidatedAddress> {
        log::info!("🔍 Validando dirección: '{}' para usuario: '{}'", address, username);

        // 🆕 PASO 0: Verificar si es una dirección incompleta
        let (preprocessed_address, warnings) = self.handle_incomplete_address(address, username);
        
        // 🎯 INTENTO 1: Dirección original (o preprocesada)
        if let Ok(result) = self.geocoding_service.geocode_address(&preprocessed_address).await {
            if self.is_valid_result(&result) {
                log::info!("✅ Dirección original válida: {}", address);
                return Ok(ValidatedAddress {
                    success: true,
                    latitude: result.latitude,
                    longitude: result.longitude,
                    formatted_address: result.formatted_address,
                    original_address: address.to_string(),
                    validation_method: ValidationMethod::Original,
                    confidence: ValidationConfidence::High,
                    warnings,
                    error: None,
                });
            }
        }

        // 🧹 INTENTO 2: Limpiar dirección (quitar nombre cliente)
        let cleaned_address = self.clean_address(address);
        if cleaned_address != address {
            if let Ok(result) = self.geocoding_service.geocode_address(&cleaned_address).await {
                if self.is_valid_result(&result) {
                    log::info!("✅ Dirección limpiada válida: {} -> {}", address, cleaned_address);
                    return Ok(ValidatedAddress {
                        success: true,
                        latitude: result.latitude,
                        longitude: result.longitude,
                        formatted_address: result.formatted_address,
                        original_address: address.to_string(),
                        validation_method: ValidationMethod::Cleaned,
                        confidence: ValidationConfidence::Medium,
                        warnings: vec!["Dirección limpiada automáticamente".to_string()],
                        error: None,
                    });
                }
            }
        }

        // 🏢 INTENTO 3: Completar con sector del username
        let sector_address = self.complete_with_sector(&cleaned_address, username);
        if sector_address != cleaned_address {
            if let Ok(result) = self.geocoding_service.geocode_address(&sector_address).await {
                if self.is_valid_result(&result) {
                    log::info!("✅ Dirección completada con sector válida: {} -> {}", address, sector_address);
                    return Ok(ValidatedAddress {
                        success: true,
                        latitude: result.latitude,
                        longitude: result.longitude,
                        formatted_address: result.formatted_address,
                        original_address: address.to_string(),
                        validation_method: ValidationMethod::CompletedWithSector,
                        confidence: ValidationConfidence::Medium,
                        warnings: vec!["Dirección completada con sector automáticamente".to_string()],
                        error: None,
                    });
                }
            }
        }

        // 🔍 INTENTO 4: Búsqueda parcial (solo calle + distrito)
        let partial_address = self.extract_street_and_district(&cleaned_address, username);
        if partial_address != cleaned_address {
            if let Ok(result) = self.geocoding_service.geocode_address(&partial_address).await {
                if self.is_valid_result(&result) {
                    log::info!("✅ Dirección encontrada por búsqueda parcial: {} -> {}", address, partial_address);
                    return Ok(ValidatedAddress {
                        success: true,
                        latitude: result.latitude,
                        longitude: result.longitude,
                        formatted_address: result.formatted_address,
                        original_address: address.to_string(),
                        validation_method: ValidationMethod::PartialSearch,
                        confidence: ValidationConfidence::Low,
                        warnings: vec!["Dirección encontrada por búsqueda parcial".to_string()],
                        error: None,
                    });
                }
            }
        }

        // ❌ FALLO TOTAL: Requiere intervención manual
        log::warn!("❌ No se pudo validar automáticamente: {}", address);
        Ok(ValidatedAddress {
            success: false,
            latitude: None,
            longitude: None,
            formatted_address: None,
            original_address: address.to_string(),
            validation_method: ValidationMethod::ManualRequired,
            confidence: ValidationConfidence::None,
            warnings: vec![],
            error: Some("No se pudo validar automáticamente. Requiere verificación manual.".to_string()),
        })
    }

    /// Validación en lote de múltiples direcciones
    pub async fn validate_addresses_batch(
        &self,
        addresses: Vec<String>,
        username: &str,
    ) -> Result<AddressValidationResult> {
        let total_addresses = addresses.len();
        log::info!("🔍 Validando {} direcciones en lote para usuario: '{}'", total_addresses, username);

        let mut validated_addresses = Vec::new();
        let mut auto_validated = 0;
        let mut cleaned_auto = 0;
        let mut completed_auto = 0;
        let mut partial_found = 0;
        let mut requires_manual = 0;
        let mut warnings = Vec::new();

        for address in addresses {
            match self.validate_address(&address, username).await {
                Ok(validated) => {
                    match validated.validation_method {
                        ValidationMethod::Original => auto_validated += 1,
                        ValidationMethod::Cleaned => cleaned_auto += 1,
                        ValidationMethod::CompletedWithSector => completed_auto += 1,
                        ValidationMethod::PartialSearch => partial_found += 1,
                        ValidationMethod::ManualRequired => requires_manual += 1,
                    }

                    if !validated.warnings.is_empty() {
                        warnings.extend(validated.warnings.clone());
                    }

                    validated_addresses.push(validated);
                }
                Err(e) => {
                    log::error!("❌ Error validando dirección '{}': {}", address, e);
                    requires_manual += 1;
                    validated_addresses.push(ValidatedAddress {
                        success: false,
                        latitude: None,
                        longitude: None,
                        formatted_address: None,
                        original_address: address,
                        validation_method: ValidationMethod::ManualRequired,
                        confidence: ValidationConfidence::None,
                        warnings: vec![],
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        // Generar resumen de warnings
        if cleaned_auto > 0 {
            warnings.push(format!("{} direcciones limpiadas automáticamente", cleaned_auto));
        }
        if completed_auto > 0 {
            warnings.push(format!("{} direcciones completadas con sector", completed_auto));
        }
        if partial_found > 0 {
            warnings.push(format!("{} direcciones encontradas por búsqueda parcial", partial_found));
        }

        Ok(AddressValidationResult {
            total_addresses,
            auto_validated,
            cleaned_auto,
            completed_auto,
            partial_found,
            requires_manual,
            validated_addresses,
            warnings,
        })
    }

    /// Verificar si un resultado de geocoding es válido
    fn is_valid_result(&self, result: &GeocodingResponse) -> bool {
        result.success && 
        result.latitude.is_some() && 
        result.longitude.is_some() &&
        result.latitude.unwrap() != 0.0 &&
        result.longitude.unwrap() != 0.0
    }

    /// Limpiar dirección removiendo nombres de clientes comunes
    fn clean_address(&self, address: &str) -> String {
        let mut cleaned = address.to_uppercase();
        let mut warnings = Vec::new();
        
        // 🆕 PASO 1: Detectar y corregir números al final
        cleaned = self.fix_number_at_end(&cleaned);
        
        // 🆕 PASO 2: Detectar y corregir números duplicados (ej: "35 35 RUE")
        if let Some(captures) = self.duplicate_number_regex.captures(cleaned.as_str()) {
            if let Some(first_number) = captures.get(1) {
                let number = first_number.as_str().to_string();
                let pattern = format!("{} {}", number, number);
                cleaned = cleaned.replace(&pattern, &number);
                warnings.push(format!("Números duplicados detectados y corregidos: {} {}", number, number));
            }
        }
        
        // 🆕 PASO 3: Detectar y corregir números separados (ej: "6 7 IMP" -> "7 IMP")
        if let Some(captures) = self.separated_numbers_regex.captures(cleaned.as_str()) {
            if let Some(first_num) = captures.get(1) {
                if let Some(second_num) = captures.get(2) {
                    let first = first_num.as_str().to_string();
                    let second = second_num.as_str().to_string();
                    // Tomar el último número (segundo)
                    let pattern = format!("{} {}", first, second);
                    cleaned = cleaned.replace(&pattern, &second);
                    warnings.push(format!("Números separados detectados ({} {}), tomando el último: {}", first, second, second));
                }
            }
        }
        
        // 🆕 PASO 4: Remover distrito del medio (ej: "18EME ARRONDISSEMENT")
        if let Some(captures) = self.district_in_middle_regex.captures(cleaned.as_str()) {
            if let Some(district) = captures.get(1) {
                let district_str = district.as_str().to_string();
                cleaned = cleaned.replace(&district_str, "");
                warnings.push(format!("Distrito en medio removido: {}", district_str));
            }
        }
        
        // 🆕 PASO 5: Usar regex para extraer solo la calle
        if let Some(captures) = self.street_regex.captures(&cleaned) {
            if let Some(street_type) = captures.get(1) {
                if let Some(street_name) = captures.get(2) {
                    let street_type_str = street_type.as_str();
                    let street_name_str = street_name.as_str();
                    
                    // Remover nombres de clientes del nombre de la calle
                    let mut clean_street_name = street_name_str.to_string();
                    for name in &self.client_names {
                        clean_street_name = clean_street_name.replace(name, "");
                    }
                    
                    // Limpiar espacios extra
                    clean_street_name = clean_street_name
                        .split_whitespace()
                        .collect::<Vec<&str>>()
                        .join(" ");
                    
                    // Si el nombre de la calle no está vacío, usar solo la calle
                    if !clean_street_name.trim().is_empty() {
                        cleaned = format!("{} {}", street_type_str, clean_street_name);
                    }
                }
            }
        }
        
        // 🆕 PASO 6: Si no se pudo extraer con regex, usar método anterior
        if cleaned == address.to_uppercase() {
            // Remover nombres comunes de clientes
            for name in &self.client_names {
                cleaned = cleaned.replace(name, "");
            }
        }
        
        // Limpiar espacios extra y caracteres especiales
        cleaned = cleaned
            .replace("  ", " ")
            .replace(" ,", ",")
            .replace(", ", ",")
            .trim()
            .to_string();
        
        // Si la dirección está muy vacía, devolver la original
        if cleaned.len() < 10 {
            address.to_string()
        } else {
            cleaned
        }
    }

    /// Completar dirección con información del sector
    fn complete_with_sector(&self, address: &str, username: &str) -> String {
        // Extraer sector del username
        let _sector = self.extract_sector_from_username(username);
        let district = self.extract_district_from_username(username);
        
        // Si la dirección no contiene el distrito, agregarlo
        if !address.to_uppercase().contains(&district.to_uppercase()) {
            format!("{}, {}", address, district)
        } else {
            address.to_string()
        }
    }

    /// Extraer calle y distrito para búsqueda parcial
    fn extract_street_and_district(&self, address: &str, username: &str) -> String {
        let district = self.extract_district_from_username(username);
        
        // Intentar extraer solo la calle (antes de la primera coma)
        if let Some(comma_pos) = address.find(',') {
            let street = &address[..comma_pos];
            format!("{}, {}", street.trim(), district)
        } else {
            // Si no hay coma, usar toda la dirección con el distrito
            format!("{}, {}", address, district)
        }
    }

    /// Extraer sector del username
    fn extract_sector_from_username(&self, username: &str) -> String {
        // Mapeo directo si existe
        if let Some(sector) = self.sector_mapping.get(username) {
            return sector.clone();
        }
        
        // 🆕 PARSING INTELIGENTE: TEST_USER -> T (sector) + 1234 (código postal)
        if username.len() >= 6 {
            let sector_letter = &username[0..1]; // A
            let postal_code_part = &username[2..]; // 7518
            
            // Formar código postal válido: 7518 -> 75018
            if postal_code_part.len() == 4 {
                let postal_code = format!("75{}", postal_code_part); // 75018
                return format!("{}{}", sector_letter, postal_code); // A75018
            }
        }
        
        "A75018".to_string() // Default
    }

    /// Extraer distrito del username
    fn extract_district_from_username(&self, username: &str) -> String {
        // 🆕 PARSING INTELIGENTE: TEST_USER -> 75018 Paris
        if username.len() >= 6 {
            let postal_code_part = &username[2..]; // 7518
            
            // Formar código postal válido: 7518 -> 75018
            if postal_code_part.len() == 4 {
                let postal_code = format!("75{}", postal_code_part); // 75018
                return format!("{} Paris", postal_code);
            }
        }
        
        "75018 Paris".to_string() // Default
    }

    /// 🆕 Corregir números al final de la dirección
    /// Ejemplo: "Rue Jean Cottin 3" -> "3 Rue Jean Cottin"
    fn fix_number_at_end(&self, address: &str) -> String {
        if let Some(captures) = self.number_regex.captures(address) {
            if let Some(rest) = captures.get(1) {
                if let Some(number) = captures.get(2) {
                    let rest_str = rest.as_str().trim();
                    let number_str = number.as_str();
                    
                    // Verificar si el resto contiene una palabra de calle
                    if self.street_regex.is_match(rest_str) {
                        // Reorganizar: "Rue Jean Cottin 3" -> "3 Rue Jean Cottin"
                        return format!("{} {}", number_str, rest_str);
                    }
                }
            }
        }
        
        address.to_string()
    }

    /// 🆕 Manejar direcciones incompletas (ej: "75, 75018 PARIS")
    fn handle_incomplete_address(&self, address: &str, username: &str) -> (String, Vec<String>) {
        let mut warnings = Vec::new();
        
        if let Some(captures) = self.incomplete_address_regex.captures(address) {
            if let Some(number) = captures.get(1) {
                if let Some(postal_code) = captures.get(2) {
                    let num = number.as_str();
                    let code = postal_code.as_str();
                    
                    // Extraer distrito del username para completar
                    let district = self.extract_district_from_username(username);
                    
                    // Intentar completar con información del sector
                    let completed = format!("{} RUE INCONNUE, {} PARIS", num, code);
                    
                    warnings.push(format!("Dirección incompleta detectada: '{}', completada con 'RUE INCONNUE'", address));
                    warnings.push(format!("Usar información del sector: {}", district));
                    
                    return (completed, warnings);
                }
            }
        }
        
        (address.to_string(), warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::geocoding_service::GeocodingService;

    #[test]
    fn test_clean_address() {
        let service = GeocodingService::new("test_token".to_string());
        let validator = AddressValidator::new(service);
        
        // Test limpieza básica
        assert_eq!(
            validator.clean_address("MARTIN 15 Rue de la Paix, 75001 Paris"),
            "15 RUE DE LA PAIX, 75001 PARIS"
        );
        
        // Test limpieza con regex
        assert_eq!(
            validator.clean_address("DUBOIS 25 Avenue des Champs, 75008 Paris"),
            "25 AVENUE DES CHAMPS, 75008 PARIS"
        );
        
        // Test corrección de número al final
        assert_eq!(
            validator.clean_address("Rue Jean Cottin 3"),
            "3 RUE JEAN COTTIN"
        );
        
        // Test extracción de calle con regex
        assert_eq!(
            validator.clean_address("MARTIN Rue de la République, 75001 Paris"),
            "RUE DE LA RÉPUBLIQUE, 75001 PARIS"
        );
    }

    #[test]
    fn test_extract_sector_from_username() {
        let service = GeocodingService::new("test_token".to_string());
        let validator = AddressValidator::new(service);
        
        // Test parsing inteligente del username
        assert_eq!(validator.extract_sector_from_username("TEST_USER"), "T75018");
        assert_eq!(validator.extract_sector_from_username("TEST_USER2"), "T75019");
        assert_eq!(validator.extract_sector_from_username("B123456"), "B75123");
    }

    #[test]
    fn test_extract_district_from_username() {
        let service = GeocodingService::new("test_token".to_string());
        let validator = AddressValidator::new(service);
        
        // Test parsing inteligente del distrito
        assert_eq!(validator.extract_district_from_username("TEST_USER"), "75018 Paris");
        assert_eq!(validator.extract_district_from_username("TEST_USER2"), "75019 Paris");
        assert_eq!(validator.extract_district_from_username("B123456"), "75123 Paris");
    }

    #[test]
    fn test_fix_number_at_end() {
        let service = GeocodingService::new("test_token".to_string());
        let validator = AddressValidator::new(service);
        
        // Test corrección de número al final
        assert_eq!(validator.fix_number_at_end("Rue Jean Cottin 3"), "3 Rue Jean Cottin");
        assert_eq!(validator.fix_number_at_end("Avenue des Champs 25"), "25 Avenue des Champs");
        assert_eq!(validator.fix_number_at_end("15 Rue de la Paix"), "15 Rue de la Paix"); // No cambia
    }

    #[test]
    fn test_handle_incomplete_address() {
        let service = GeocodingService::new("test_token".to_string());
        let validator = AddressValidator::new(service);
        
        // Test dirección incompleta
        let (result, warnings) = validator.handle_incomplete_address("75, 75018 PARIS", "TEST_USER");
        assert!(result.contains("RUE INCONNUE"));
        assert!(warnings.len() > 0);
        assert!(warnings[0].contains("Dirección incompleta detectada"));
    }

    #[test]
    fn test_clean_address_improvements() {
        let service = GeocodingService::new("test_token".to_string());
        let validator = AddressValidator::new(service);
        
        // Test números duplicados
        assert_eq!(
            validator.clean_address("35 35 RUE MARC SEGUIN"),
            "35 RUE MARC SEGUIN"
        );
        
        // Test números separados (tomar el último)
        assert_eq!(
            validator.clean_address("6 7 IMP. DU CURE"),
            "7 IMP. DU CURE"
        );
        
        // Test distrito en medio
        assert_eq!(
            validator.clean_address("16 RUE JEAN COTTIN 18EME ARRONDISSEMENT"),
            "16 RUE JEAN COTTIN"
        );
    }
}
