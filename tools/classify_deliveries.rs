use std::{env, fs, path::Path};

mod delivery_classifier {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Serialize, Clone)]
    pub enum DeliveryType {
        Relais,      // Point Relais
        RelaisDirect, // Point Relais Directo
        RCS,         // Empresa (Prioritaria)
        Domicile,    // Entrega a domicilio
        Unknown,     // Tipo desconocido
    }

    #[derive(Debug, Serialize, Clone)]
    pub struct RelaisData {
        pub name: String,           // complementAdresse1OrigineDestinataire
        pub address: String,        // LibelleVoieGeocodeDestinataire
        pub coordinates: (f64, f64), // coordX/YDestinataire
        pub barcode: String,        // codeBarreArticle
        pub phone: String,          // telephoneMobileDestinataire
        pub postal_code: String,    // codePostalGeocodeDestinataire
        pub city: String,           // LibelleLocaliteGeocodeDestinataire
    }

    #[derive(Debug, Serialize, Clone)]
    pub struct RCSData {
        pub company: String,        // nomDestinataire
        pub address: String,        // LibelleVoieGeocodeDestinataire
        pub coordinates: (f64, f64), // coordX/YDestinataire
        pub barcode: String,        // codeBarreArticle
        pub phone: String,          // telephoneMobileDestinataire
        pub email: String,          // emailDestinataire
        pub postal_code: String,    // codePostalGeocodeDestinataire
        pub city: String,           // LibelleLocaliteGeocodeDestinataire
    }

    #[derive(Debug, Serialize, Clone)]
    pub struct DomicileData {
        pub original_address: String,    // LibelleVoieOrigineDestinataire
        pub geocoded_address: String,    // LibelleVoieGeocodeDestinataire
        pub delivery_details: String,    // Extraído: "PORTE 106", "APT 3B", etc.
        pub coordinates: (f64, f64),     // coordX/YDestinataire
        pub barcode: String,            // codeBarreArticle
        pub customer: CustomerContact,   // Datos del cliente
        pub postal_code: String,        // codePostalGeocodeDestinataire
        pub city: String,               // LibelleLocaliteGeocodeDestinataire
    }

    #[derive(Debug, Serialize, Clone)]
    pub struct CustomerContact {
        pub name: String,            // nomDestinataire
        pub phone: String,           // telephoneMobileDestinataire
        pub email: Option<String>,   // emailDestinataire (filtrado)
        pub phone_fixe: Option<String>, // telephoneFixeDestinataire
    }

    impl CustomerContact {
        pub fn is_valid_email(&self) -> bool {
            if let Some(email) = &self.email {
                !email.contains("aliexpress") && 
                !email.contains("temu") && 
                !email.contains("tiktok") && 
                !email.contains("amazon") && 
                !email.contains("ebay") &&
                !email.contains("google_") &&
                email.contains("@")
            } else {
                false
            }
        }
    }

    #[derive(Debug, Serialize, Clone)]
    pub struct DeliveryPackage {
        pub id: String,
        pub delivery_type: DeliveryType,
        pub relais_data: Option<RelaisData>,
        pub rcs_data: Option<RCSData>,
        pub domicile_data: Option<DomicileData>,
        pub geocoding_quality: String,
        pub algorithm_used: String,
    }

    impl DeliveryPackage {
        pub fn new(id: String) -> Self {
            DeliveryPackage {
                id,
                delivery_type: DeliveryType::Unknown,
                relais_data: None,
                rcs_data: None,
                domicile_data: None,
                geocoding_quality: "Unknown".to_string(),
                algorithm_used: "Unknown".to_string(),
            }
        }

        pub fn classify_from_json(&mut self, json_data: &serde_json::Value) {
            // Extraer tipo de livraison
            if let Some(type_livraison) = json_data.get("typeLivraison").and_then(|v| v.as_str()) {
                self.delivery_type = match type_livraison {
                    "RELAIS" => DeliveryType::Relais,
                    "RELAISDIRECT" => DeliveryType::RelaisDirect,
                    "RCS" => DeliveryType::RCS,
                    "DOMICILE" => DeliveryType::Domicile,
                    _ => DeliveryType::Unknown,
                };
            }

            // Extraer calidad de geocoding
            if let Some(quality) = json_data.get("qualiteGeocodageDestinataire").and_then(|v| v.as_str()) {
                self.geocoding_quality = quality.to_string();
            }

            // Extraer algoritmo usado
            if let Some(algo) = json_data.get("AlgoSolrDestinataire").and_then(|v| v.as_str()) {
                self.algorithm_used = algo.to_string();
            }

            // Clasificar y extraer datos según el tipo
            match self.delivery_type {
                DeliveryType::Relais | DeliveryType::RelaisDirect => {
                    self.extract_relais_data(json_data);
                },
                DeliveryType::RCS => {
                    self.extract_rcs_data(json_data);
                },
                DeliveryType::Domicile => {
                    self.extract_domicile_data(json_data);
                },
                DeliveryType::Unknown => {
                    // Intentar clasificar por otros campos
                    self.try_classify_by_other_fields(json_data);
                }
            }
        }

        fn extract_relais_data(&mut self, json_data: &serde_json::Value) {
            let name = json_data.get("complementAdresse1OrigineDestinataire")
                .and_then(|v| v.as_str())
                .unwrap_or("Point Relais")
                .to_string();

            let address = json_data.get("LibelleVoieGeocodeDestinataire")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let coordinates = (
                json_data.get("coordXDestinataire").and_then(|v| v.as_f64()).unwrap_or(0.0),
                json_data.get("coordYDestinataire").and_then(|v| v.as_f64()).unwrap_or(0.0),
            );

            let barcode = json_data.get("codeBarreArticle")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let phone = json_data.get("telephoneMobileDestinataire")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let postal_code = json_data.get("codePostalGeocodeDestinataire")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let city = json_data.get("LibelleLocaliteGeocodeDestinataire")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            self.relais_data = Some(RelaisData {
                name,
                address,
                coordinates,
                barcode,
                phone,
                postal_code,
                city,
            });
        }

        fn extract_rcs_data(&mut self, json_data: &serde_json::Value) {
            let company = json_data.get("nomDestinataire")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let address = json_data.get("LibelleVoieGeocodeDestinataire")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let coordinates = (
                json_data.get("coordXDestinataire").and_then(|v| v.as_f64()).unwrap_or(0.0),
                json_data.get("coordYDestinataire").and_then(|v| v.as_f64()).unwrap_or(0.0),
            );

            let barcode = json_data.get("codeBarreArticle")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let phone = json_data.get("telephoneMobileDestinataire")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let email = json_data.get("emailDestinataire")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let postal_code = json_data.get("codePostalGeocodeDestinataire")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let city = json_data.get("LibelleLocaliteGeocodeDestinataire")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            self.rcs_data = Some(RCSData {
                company,
                address,
                coordinates,
                barcode,
                phone,
                email,
                postal_code,
                city,
            });
        }

        fn extract_domicile_data(&mut self, json_data: &serde_json::Value) {
            let original_address = json_data.get("LibelleVoieOrigineDestinataire")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let geocoded_address = json_data.get("LibelleVoieGeocodeDestinataire")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Extraer detalles de entrega
            let delivery_details = extract_delivery_details(&original_address, &geocoded_address);

            let coordinates = (
                json_data.get("coordXDestinataire").and_then(|v| v.as_f64()).unwrap_or(0.0),
                json_data.get("coordYDestinataire").and_then(|v| v.as_f64()).unwrap_or(0.0),
            );

            let barcode = json_data.get("codeBarreArticle")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let customer = CustomerContact {
                name: json_data.get("nomDestinataire")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                phone: json_data.get("telephoneMobileDestinataire")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                email: json_data.get("emailDestinataire")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                phone_fixe: json_data.get("telephoneFixeDestinataire")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            };

            let postal_code = json_data.get("codePostalGeocodeDestinataire")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let city = json_data.get("LibelleLocaliteGeocodeDestinataire")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            self.domicile_data = Some(DomicileData {
                original_address,
                geocoded_address,
                delivery_details,
                coordinates,
                barcode,
                customer,
                postal_code,
                city,
            });
        }

        fn try_classify_by_other_fields(&mut self, _json_data: &serde_json::Value) {
            // Lógica para clasificar por otros campos si typeLivraison no está disponible
            // Por ahora mantenemos Unknown
        }
    }

    // Función para extraer detalles de entrega
    fn extract_delivery_details(original: &str, geocoded: &str) -> String {
        if original.len() <= geocoded.len() {
            return String::new();
        }

        // Buscar patrones comunes de detalles de entrega
        let patterns = vec![
            "PORTE", "APT", "ETAGE", "ESCALIER", "BATIMENT", "RESIDENCE",
            "IMMEUBLE", "TOUR", "BLOC", "COULOIR", "COUR", "JARDIN"
        ];

        let mut details = Vec::new();
        let original_upper = original.to_uppercase();
        let geocoded_upper = geocoded.to_uppercase();

        for pattern in patterns {
            if let Some(pos) = original_upper.find(pattern) {
                if !geocoded_upper.contains(&original_upper[pos..pos + pattern.len()]) {
                    // Extraer desde el patrón hasta el final o hasta el siguiente espacio
                    let remaining = &original[pos..];
                    if let Some(space_pos) = remaining.find(' ') {
                        details.push(remaining[..space_pos].to_string());
                    } else {
                        details.push(remaining.to_string());
                    }
                }
            }
        }

        details.join(" ")
    }

    #[derive(Debug, Serialize)]
    pub struct DeliveryClassificationReport {
        pub total_packages: usize,
        pub relais_count: usize,
        pub relais_direct_count: usize,
        pub rcs_count: usize,
        pub domicile_count: usize,
        pub unknown_count: usize,
        pub packages_with_coordinates: usize,
        pub packages_with_valid_emails: usize,
        pub packages_with_phones: usize,
        pub quality_distribution: HashMap<String, usize>,
        pub algorithm_distribution: HashMap<String, usize>,
        pub packages: Vec<DeliveryPackage>,
    }

    impl DeliveryClassificationReport {
        pub fn new() -> Self {
            DeliveryClassificationReport {
                total_packages: 0,
                relais_count: 0,
                relais_direct_count: 0,
                rcs_count: 0,
                domicile_count: 0,
                unknown_count: 0,
                packages_with_coordinates: 0,
                packages_with_valid_emails: 0,
                packages_with_phones: 0,
                quality_distribution: HashMap::new(),
                algorithm_distribution: HashMap::new(),
                packages: Vec::new(),
            }
        }

        pub fn add_package(&mut self, package: DeliveryPackage) {
            self.total_packages += 1;

            match package.delivery_type {
                DeliveryType::Relais => self.relais_count += 1,
                DeliveryType::RelaisDirect => self.relais_direct_count += 1,
                DeliveryType::RCS => self.rcs_count += 1,
                DeliveryType::Domicile => self.domicile_count += 1,
                DeliveryType::Unknown => self.unknown_count += 1,
            }

            // Verificar coordenadas
            let has_coordinates = match &package.delivery_type {
                DeliveryType::Relais | DeliveryType::RelaisDirect => {
                    package.relais_data.as_ref()
                        .map(|r| r.coordinates.0 != 0.0 || r.coordinates.1 != 0.0)
                        .unwrap_or(false)
                },
                DeliveryType::RCS => {
                    package.rcs_data.as_ref()
                        .map(|r| r.coordinates.0 != 0.0 || r.coordinates.1 != 0.0)
                        .unwrap_or(false)
                },
                DeliveryType::Domicile => {
                    package.domicile_data.as_ref()
                        .map(|d| d.coordinates.0 != 0.0 || d.coordinates.1 != 0.0)
                        .unwrap_or(false)
                },
                DeliveryType::Unknown => false,
            };

            if has_coordinates {
                self.packages_with_coordinates += 1;
            }

            // Verificar emails válidos
            if let Some(domicile_data) = &package.domicile_data {
                if domicile_data.customer.is_valid_email() {
                    self.packages_with_valid_emails += 1;
                }
            }

            // Verificar teléfonos
            let has_phone = match &package.delivery_type {
                DeliveryType::Relais | DeliveryType::RelaisDirect => {
                    package.relais_data.as_ref()
                        .map(|r| !r.phone.is_empty())
                        .unwrap_or(false)
                },
                DeliveryType::RCS => {
                    package.rcs_data.as_ref()
                        .map(|r| !r.phone.is_empty())
                        .unwrap_or(false)
                },
                DeliveryType::Domicile => {
                    package.domicile_data.as_ref()
                        .map(|d| !d.customer.phone.is_empty())
                        .unwrap_or(false)
                },
                DeliveryType::Unknown => false,
            };

            if has_phone {
                self.packages_with_phones += 1;
            }

            // Actualizar distribuciones
            *self.quality_distribution.entry(package.geocoding_quality.clone()).or_insert(0) += 1;
            *self.algorithm_distribution.entry(package.algorithm_used.clone()).or_insert(0) += 1;

            self.packages.push(package);
        }

        pub fn generate_report(&self) {
            println!("\n{}", "=".repeat(60));
            println!("CLASIFICACIÓN DE ENTREGAS - DELIVERY ROUTING");
            println!("{}", "=".repeat(60));

            println!("\n📦 Total de paquetes: {}", self.total_packages);

            println!("\n🚚 Distribución por Tipo de Entrega:");
            println!("  🏪 Point Relais: {} ({:.1}%)",
                    self.relais_count,
                    (self.relais_count as f64 / self.total_packages as f64) * 100.0);
            println!("  🏪 Point Relais Directo: {} ({:.1}%)",
                    self.relais_direct_count,
                    (self.relais_direct_count as f64 / self.total_packages as f64) * 100.0);
            println!("  🏢 RCS (Empresa): {} ({:.1}%)",
                    self.rcs_count,
                    (self.rcs_count as f64 / self.total_packages as f64) * 100.0);
            println!("  🏠 Domicilio: {} ({:.1}%)",
                    self.domicile_count,
                    (self.domicile_count as f64 / self.total_packages as f64) * 100.0);
            println!("  ❓ Desconocido: {} ({:.1}%)",
                    self.unknown_count,
                    (self.unknown_count as f64 / self.total_packages as f64) * 100.0);

            println!("\n📍 Datos Disponibles:");
            println!("  🗺️  Con coordenadas: {} ({:.1}%)",
                    self.packages_with_coordinates,
                    (self.packages_with_coordinates as f64 / self.total_packages as f64) * 100.0);
            println!("  📧 Con emails válidos: {} ({:.1}%)",
                    self.packages_with_valid_emails,
                    (self.packages_with_valid_emails as f64 / self.total_packages as f64) * 100.0);
            println!("  📱 Con teléfonos: {} ({:.1}%)",
                    self.packages_with_phones,
                    (self.packages_with_phones as f64 / self.total_packages as f64) * 100.0);

            println!("\n🏆 Distribución de Calidad:");
            for (quality, count) in &self.quality_distribution {
                println!("  {}: {} ({:.1}%)", quality, count, (*count as f64 / self.total_packages as f64) * 100.0);
            }

            println!("\n🤖 Distribución de Algoritmos:");
            for (algo, count) in &self.algorithm_distribution {
                println!("  {}: {} ({:.1}%)", algo, count, (*count as f64 / self.total_packages as f64) * 100.0);
            }

            // Mostrar ejemplos de cada tipo
            self.show_examples();
        }

        fn show_examples(&self) {
            println!("\n🔍 Ejemplos por Tipo:");

            // Ejemplo RELAIS
            if let Some(relais) = self.packages.iter().find(|p| matches!(p.delivery_type, DeliveryType::Relais)) {
                if let Some(data) = &relais.relais_data {
                    println!("\n  🏪 Point Relais:");
                    println!("    Nombre: {}", data.name);
                    println!("    Dirección: {}", data.address);
                    println!("    Coordenadas: ({:.5}, {:.5})", data.coordinates.0, data.coordinates.1);
                    println!("    Código de barras: {}", data.barcode);
                }
            }

            // Ejemplo RCS
            if let Some(rcs) = self.packages.iter().find(|p| matches!(p.delivery_type, DeliveryType::RCS)) {
                if let Some(data) = &rcs.rcs_data {
                    println!("\n  🏢 RCS:");
                    println!("    Empresa: {}", data.company);
                    println!("    Dirección: {}", data.address);
                    println!("    Coordenadas: ({:.5}, {:.5})", data.coordinates.0, data.coordinates.1);
                    println!("    Email: {}", data.email);
                }
            }

            // Ejemplo DOMICILE
            if let Some(domicile) = self.packages.iter().find(|p| matches!(p.delivery_type, DeliveryType::Domicile)) {
                if let Some(data) = &domicile.domicile_data {
                    println!("\n  🏠 Domicilio:");
                    println!("    Cliente: {}", data.customer.name);
                    println!("    Dirección original: {}", data.original_address);
                    println!("    Dirección geocodificada: {}", data.geocoded_address);
                    println!("    Detalles de entrega: {}", data.delivery_details);
                    println!("    Coordenadas: ({:.5}, {:.5})", data.coordinates.0, data.coordinates.1);
                    println!("    Email válido: {}", data.customer.is_valid_email());
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Uso: {} <ruta_al_json>", args[0]);
        std::process::exit(1);
    }

    let json_path = Path::new(&args[1]);

    println!("Clasificando entregas desde: {:?}", json_path);

    // Leer el JSON
    let content = fs::read_to_string(json_path)?;
    let json_data: serde_json::Value = serde_json::from_str(&content)?;

    let mut report = delivery_classifier::DeliveryClassificationReport::new();

    // Procesar cada paquete en LstLieuArticle
    if let Some(articles) = json_data.get("LstLieuArticle").and_then(|v| v.as_array()) {
        for (index, article) in articles.iter().enumerate() {
            let package_id = article.get("idLieuArticle")
                .and_then(|v| v.as_str())
                .unwrap_or(&format!("package_{}", index))
                .to_string();

            let mut package = delivery_classifier::DeliveryPackage::new(package_id);
            package.classify_from_json(article);
            report.add_package(package);
        }
    } else {
        eprintln!("No se encontró LstLieuArticle en el JSON");
        return Ok(());
    }

    // Generar reporte
    report.generate_report();

    // Guardar reporte completo en JSON
    let report_json = serde_json::to_string_pretty(&report)?;
    fs::write("delivery_classification_report.json", report_json)?;
    println!("\n💾 Reporte completo guardado en: delivery_classification_report.json");

    Ok(())
}
