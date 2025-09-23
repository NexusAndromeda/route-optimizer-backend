//! Utilidades de validación
//! 
//! Este módulo contiene funciones helper para validación de datos
//! y conversión de tipos.

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use uuid::Uuid;
use validator::ValidationError;
use serde::Serialize;

/// Validar y convertir string a UUID
pub fn validate_uuid(value: &str) -> Result<Uuid, ValidationError> {
    Uuid::parse_str(value).map_err(|_| {
        let mut error = ValidationError::new("uuid");
        error.add_param("value".into(), &value.to_string());
        error
    })
}

/// Validar y convertir string a fecha
pub fn validate_date(value: &str) -> Result<NaiveDate, ValidationError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| {
        let mut error = ValidationError::new("date");
        error.add_param("value".into(), &value.to_string());
        error.add_param("format".into(), &"YYYY-MM-DD".to_string());
        error
    })
}

/// Validar y convertir string a tiempo
pub fn validate_time(value: &str) -> Result<NaiveTime, ValidationError> {
    NaiveTime::parse_from_str(value, "%H:%M:%S").map_err(|_| {
        let mut error = ValidationError::new("time");
        error.add_param("value".into(), &value.to_string());
        error.add_param("format".into(), &"HH:MM:SS".to_string());
        error
    })
}

/// Validar y convertir string a datetime
pub fn validate_datetime(value: &str) -> Result<DateTime<Utc>, ValidationError> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| {
            let mut error = ValidationError::new("datetime");
            error.add_param("value".into(), &value.to_string());
            error.add_param("format".into(), &"RFC3339".to_string());
            error
        })
}

/// Validar que un string no esté vacío
pub fn validate_not_empty(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        let mut error = ValidationError::new("not_empty");
        error.add_param("value".into(), &value.to_string());
        return Err(error);
    }
    Ok(())
}

/// Validar longitud mínima y máxima
pub fn validate_length(value: &str, min: usize, max: usize) -> Result<(), ValidationError> {
    let len = value.chars().count();
    if len < min || len > max {
        let mut error = ValidationError::new("length");
        error.add_param("min".into(), &min);
        error.add_param("max".into(), &max);
        error.add_param("actual".into(), &len);
        return Err(error);
    }
    Ok(())
}

/// Validar que un valor esté en un rango específico
pub fn validate_range<T: PartialOrd + std::fmt::Display + serde::Serialize>(
    value: T,
    min: T,
    max: T,
) -> Result<(), ValidationError> {
    if value < min || value > max {
        let mut error = ValidationError::new("range");
        error.add_param("min".into(), &min);
        error.add_param("max".into(), &max);
        error.add_param("actual".into(), &value);
        return Err(error);
    }
    Ok(())
}

/// Validar formato de email
pub fn validate_email(value: &str) -> Result<(), ValidationError> {
    if !value.contains('@') || !value.contains('.') {
        let mut error = ValidationError::new("email");
        error.add_param("value".into(), &value.to_string());
        return Err(error);
    }
    Ok(())
}

/// Validar formato de teléfono (básico)
pub fn validate_phone(value: &str) -> Result<(), ValidationError> {
    let clean_phone = value.chars().filter(|c| c.is_digit(10)).collect::<String>();
    if clean_phone.len() < 10 || clean_phone.len() > 15 {
        let mut error = ValidationError::new("phone");
        error.add_param("value".into(), &value.to_string());
        return Err(error);
    }
    Ok(())
}

/// Validar que un valor esté en una lista de valores permitidos
pub fn validate_enum<T: PartialEq + std::fmt::Display + std::fmt::Debug + serde::Serialize>(
    value: T,
    allowed_values: &[T],
) -> Result<(), ValidationError> {
    if !allowed_values.contains(&value) {
        let mut error = ValidationError::new("enum");
        error.add_param("value".into(), &value);
        error.add_param("allowed_values".into(), &format!("{:?}", allowed_values));
        return Err(error);
    }
    Ok(())
}

/// Validar formato de coordenadas GPS (simplificado)
pub fn validate_coordinates(lat: f64, lng: f64) -> Result<(), ValidationError> {
    if lat < -90.0 || lat > 90.0 {
        let mut error = ValidationError::new("latitude");
        error.add_param("value".into(), &lat);
        error.add_param("range".into(), &"-90.0 to 90.0".to_string());
        return Err(error);
    }
    
    if lng < -180.0 || lng > 180.0 {
        let mut error = ValidationError::new("longitude");
        error.add_param("value".into(), &lng);
        error.add_param("range".into(), &"-180.0 to 180.0".to_string());
        return Err(error);
    }
    
    Ok(())
}

/// Validar que un valor sea positivo
pub fn validate_positive<T: PartialOrd + std::fmt::Display + num_traits::Zero + Serialize>(
    value: T,
) -> Result<(), ValidationError> {
    if value <= T::zero() {
        let mut error = ValidationError::new("positive");
        error.add_param("value".into(), &value);
        return Err(error);
    }
    Ok(())
}

/// Validar que un valor sea no negativo
pub fn validate_non_negative<T: PartialOrd + std::fmt::Display + num_traits::Zero + Serialize>(
    value: T,
) -> Result<(), ValidationError> {
    if value < T::zero() {
        let mut error = ValidationError::new("non_negative");
        error.add_param("value".into(), &value);
        return Err(error);
    }
    Ok(())
}

/// Validar formato de matrícula de vehículo
pub fn validate_license_plate(value: &str) -> Result<(), ValidationError> {
    // Formato básico: XX-123-XX o similar
    let clean_plate = value.replace([' ', '-', '_'], "");
    if clean_plate.len() < 5 || clean_plate.len() > 10 {
        let mut error = ValidationError::new("license_plate");
        error.add_param("value".into(), &value.to_string());
        return Err(error);
    }
    Ok(())
}

/// Validar formato de número de tournée
pub fn validate_tournee_number(value: &str) -> Result<(), ValidationError> {
    if !value.starts_with('T') || value.len() < 2 {
        let mut error = ValidationError::new("tournee_number");
        error.add_param("value".into(), &value.to_string());
        error.add_param("format".into(), &"T followed by numbers".to_string());
        return Err(error);
    }
    Ok(())
}

/// Validar formato de número de tracking
pub fn validate_tracking_number(value: &str) -> Result<(), ValidationError> {
    if value.len() < 5 || value.len() > 100 {
        let mut error = ValidationError::new("tracking_number");
        error.add_param("value".into(), &value.to_string());
        error.add_param("length".into(), &"5-100 characters".to_string());
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_uuid() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(validate_uuid(valid_uuid).is_ok());
        
        let invalid_uuid = "invalid-uuid";
        assert!(validate_uuid(invalid_uuid).is_err());
    }

    #[test]
    fn test_validate_date() {
        let valid_date = "2024-01-15";
        assert!(validate_date(valid_date).is_ok());
        
        let invalid_date = "2024/01/15";
        assert!(validate_date(invalid_date).is_err());
    }

    #[test]
    fn test_validate_length() {
        let value = "test";
        assert!(validate_length(value, 1, 10).is_ok());
        assert!(validate_length(value, 5, 10).is_err());
        assert!(validate_length(value, 1, 3).is_err());
    }

    #[test]
    fn test_validate_range() {
        assert!(validate_range(5, 1, 10).is_ok());
        assert!(validate_range(0, 1, 10).is_err());
        assert!(validate_range(15, 1, 10).is_err());
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid-email").is_err());
        assert!(validate_email("test@").is_err());
    }

    #[test]
    fn test_validate_phone() {
        assert!(validate_phone("1234567890").is_ok());
        assert!(validate_phone("123").is_err());
        assert!(validate_phone("1234567890123456").is_err());
    }

    #[test]
    fn test_validate_enum() {
        let allowed = vec!["admin", "driver"];
        assert!(validate_enum("admin", &allowed).is_ok());
        assert!(validate_enum("user", &allowed).is_err());
    }

    #[test]
    fn test_validate_coordinates() {
        assert!(validate_coordinates(45.0, -75.0).is_ok());
        assert!(validate_coordinates(91.0, -75.0).is_err());
        assert!(validate_coordinates(45.0, -181.0).is_err());
    }

    #[test]
    fn test_validate_positive() {
        assert!(validate_positive(5).is_ok());
        assert!(validate_positive(0).is_err());
        assert!(validate_positive(-5).is_err());
    }

    #[test]
    fn test_validate_license_plate() {
        assert!(validate_license_plate("AB-123-CD").is_ok());
        assert!(validate_license_plate("A").is_err());
        assert!(validate_license_plate("ABCDEFGHIJK").is_err());
    }

    #[test]
    fn test_validate_tournee_number() {
        assert!(validate_tournee_number("T001").is_ok());
        assert!(validate_tournee_number("001").is_err());
        assert!(validate_tournee_number("T").is_err());
    }

    #[test]
    fn test_validate_tracking_number() {
        assert!(validate_tracking_number("TRK123456").is_ok());
        assert!(validate_tracking_number("123").is_err());
        assert!(validate_tracking_number(&"A".repeat(101)).is_err());
    }
}

