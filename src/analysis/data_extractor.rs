use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone)]
pub struct OptimizedRelaisData {
    pub name: String,                    // Nombre del Point Relais
    pub address: String,                 // Dirección geocodificada
    pub coordinates: (f64, f64),         // Coordenadas GPS
    pub barcode: String,                 // Código de barras
    pub phone: String,                   // Teléfono de contacto
    pub postal_code: String,             // Código postal
    pub city: String,                    // Ciudad
    pub delivery_type: String,           // RELAIS o RELAISDIRECT
    pub geocoding_quality: String,       // Calidad del geocoding
    pub algorithm_used: String,          // Algoritmo usado
    pub ready_for_routing: bool,         // Listo para routing
}

#[derive(Debug, Serialize, Clone)]
pub struct OptimizedRCSData {
    pub company: String,                 // Nombre de la empresa
    pub contact_person: String,          // Persona de contacto
    pub address: String,                 // Dirección geocodificada
    pub coordinates: (f64, f64),         // Coordenadas GPS
    pub barcode: String,                 // Código de barras
    pub phone: String,                   // Teléfono de contacto
    pub email: String,                   // Email de contacto
    pub postal_code: String,             // Código postal
    pub city: String,                    // Ciudad
    pub geocoding_quality: String,       // Calidad del geocoding
    pub algorithm_used: String,          // Algoritmo usado
    pub ready_for_routing: bool,         // Listo para routing
    pub is_priority: bool,               // Es entrega prioritaria
}

#[derive(Debug, Serialize, Clone)]
pub struct OptimizedDomicileData {
    pub customer: OptimizedCustomerContact, // Datos del cliente
    pub original_address: String,           // Dirección original completa
    pub geocoded_address: String,           // Dirección geocodificada
    pub delivery_details: DeliveryDetails,  // Detalles de entrega extraídos
    pub coordinates: (f64, f64),            // Coordenadas GPS
    pub barcode: String,                    // Código de barras
    pub postal_code: String,                // Código postal
    pub city: String,                       // Ciudad
    pub geocoding_quality: String,          // Calidad del geocoding
    pub algorithm_used: String,             // Algoritmo usado
    pub ready_for_routing: bool,            // Listo para routing
    pub needs_validation: bool,             // Necesita validación
    pub delivery_instructions: String,      // Instrucciones de entrega
}

#[derive(Debug, Serialize, Clone)]
pub struct OptimizedCustomerContact {
    pub name: String,                       // Nombre del cliente
    pub phone: String,                      // Teléfono móvil
    pub phone_fixe: Option<String>,         // Teléfono fijo
    pub email: Option<String>,              // Email (filtrado)
    pub email_valid: bool,                  // Email es válido
    pub can_receive_sms: bool,              // Puede recibir SMS
    pub can_receive_email: bool,            // Puede recibir email
    pub can_receive_whatsapp: bool,         // Puede recibir WhatsApp
}

#[derive(Debug, Serialize, Clone)]
pub struct DeliveryDetails {
    pub porte: Option<String>,              // PORTE 106
    pub apt: Option<String>,                // APT 3B
    pub etage: Option<String>,              // ETAGE 2
    pub escalier: Option<String>,           // ESCALIER A
    pub batiment: Option<String>,           // BATIMENT 1
    pub residence: Option<String>,          // RESIDENCE LES ROSES
    pub immeuble: Option<String>,           // IMMEUBLE B
    pub tour: Option<String>,               // TOUR 1
    pub bloc: Option<String>,               // BLOC A
    pub couloir: Option<String>,            // COULOIR 2
    pub cour: Option<String>,               // COUR INTERIEURE
    pub jardin: Option<String>,             // JARDIN
    pub other: Vec<String>,                 // Otros detalles
}

impl DeliveryDetails {
    pub fn new() -> Self {
        DeliveryDetails {
            porte: None,
            apt: None,
            etage: None,
            escalier: None,
            batiment: None,
            residence: None,
            immeuble: None,
            tour: None,
            bloc: None,
            couloir: None,
            cour: None,
            jardin: None,
            other: Vec::new(),
        }
    }

    pub fn to_string(&self) -> String {
        let mut details = Vec::new();
        
        if let Some(porte) = &self.porte { details.push(porte.clone()); }
        if let Some(apt) = &self.apt { details.push(apt.clone()); }
        if let Some(etage) = &self.etage { details.push(etage.clone()); }
        if let Some(escalier) = &self.escalier { details.push(escalier.clone()); }
        if let Some(batiment) = &self.batiment { details.push(batiment.clone()); }
        if let Some(residence) = &self.residence { details.push(residence.clone()); }
        if let Some(immeuble) = &self.immeuble { details.push(immeuble.clone()); }
        if let Some(tour) = &self.tour { details.push(tour.clone()); }
        if let Some(bloc) = &self.bloc { details.push(bloc.clone()); }
        if let Some(couloir) = &self.couloir { details.push(couloir.clone()); }
        if let Some(cour) = &self.cour { details.push(cour.clone()); }
        if let Some(jardin) = &self.jardin { details.push(jardin.clone()); }
        
        details.extend(self.other.clone());
        details.join(" ")
    }

    pub fn is_empty(&self) -> bool {
        self.porte.is_none() && self.apt.is_none() && self.etage.is_none() &&
        self.escalier.is_none() && self.batiment.is_none() && self.residence.is_none() &&
        self.immeuble.is_none() && self.tour.is_none() && self.bloc.is_none() &&
        self.couloir.is_none() && self.cour.is_none() && self.jardin.is_none() &&
        self.other.is_empty()
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct OptimizedDeliveryPackage {
    pub id: String,
    pub package_type: String,              // RELAIS, RELAISDIRECT, RCS, DOMICILE
    pub relais_data: Option<OptimizedRelaisData>,
    pub rcs_data: Option<OptimizedRCSData>,
    pub domicile_data: Option<OptimizedDomicileData>,
    pub processing_time_ms: u64,           // Tiempo de procesamiento
    pub extraction_quality: String,        // Calidad de extracción
}

impl OptimizedDeliveryPackage {
    pub fn new(id: String) -> Self {
        OptimizedDeliveryPackage {
            id,
            package_type: "UNKNOWN".to_string(),
            relais_data: None,
            rcs_data: None,
            domicile_data: None,
            processing_time_ms: 0,
            extraction_quality: "UNKNOWN".to_string(),
        }
    }

    pub fn extract_from_json(&mut self, json_data: &serde_json::Value) -> Result<(), String> {
        let start_time = std::time::Instant::now();

        // Extraer tipo de livraison
        let type_livraison = json_data.get("typeLivraison")
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN");

        self.package_type = type_livraison.to_string();

        // Clasificar y extraer según el tipo
        match type_livraison {
            "RELAIS" | "RELAISDIRECT" => {
                self.extract_relais_data(json_data)?;
            },
            "RCS" => {
                self.extract_rcs_data(json_data)?;
            },
            "DOMICILE" => {
                self.extract_domicile_data(json_data)?;
            },
            _ => {
                return Err(format!("Tipo de livraison desconocido: {}", type_livraison));
            }
        }

        self.processing_time_ms = start_time.elapsed().as_millis() as u64;
        self.extraction_quality = "SUCCESS".to_string();

        Ok(())
    }

    fn extract_relais_data(&mut self, json_data: &serde_json::Value) -> Result<(), String> {
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

        let geocoding_quality = json_data.get("qualiteGeocodageDestinataire")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let algorithm_used = json_data.get("AlgoSolrDestinataire")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let ready_for_routing = coordinates.0 != 0.0 || coordinates.1 != 0.0;

        self.relais_data = Some(OptimizedRelaisData {
            name,
            address,
            coordinates,
            barcode,
            phone,
            postal_code,
            city,
            delivery_type: self.package_type.clone(),
            geocoding_quality,
            algorithm_used,
            ready_for_routing,
        });

        Ok(())
    }

    fn extract_rcs_data(&mut self, json_data: &serde_json::Value) -> Result<(), String> {
        let company = json_data.get("nomDestinataire")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let contact_person = company.clone(); // En RCS, el destinatario es la empresa

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

        let geocoding_quality = json_data.get("qualiteGeocodageDestinataire")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let algorithm_used = json_data.get("AlgoSolrDestinataire")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let ready_for_routing = coordinates.0 != 0.0 || coordinates.1 != 0.0;
        let is_priority = true; // RCS siempre es prioritario

        self.rcs_data = Some(OptimizedRCSData {
            company,
            contact_person,
            address,
            coordinates,
            barcode,
            phone,
            email,
            postal_code,
            city,
            geocoding_quality,
            algorithm_used,
            ready_for_routing,
            is_priority,
        });

        Ok(())
    }

    fn extract_domicile_data(&mut self, json_data: &serde_json::Value) -> Result<(), String> {
        let original_address = json_data.get("LibelleVoieOrigineDestinataire")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let geocoded_address = json_data.get("LibelleVoieGeocodeDestinataire")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Extraer detalles de entrega optimizados
        let delivery_details = extract_delivery_details_optimized(&original_address, &geocoded_address);

        let coordinates = (
            json_data.get("coordXDestinataire").and_then(|v| v.as_f64()).unwrap_or(0.0),
            json_data.get("coordYDestinataire").and_then(|v| v.as_f64()).unwrap_or(0.0),
        );

        let barcode = json_data.get("codeBarreArticle")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let customer = extract_customer_contact_optimized(json_data);

        let postal_code = json_data.get("codePostalGeocodeDestinataire")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let city = json_data.get("LibelleLocaliteGeocodeDestinataire")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let geocoding_quality = json_data.get("qualiteGeocodageDestinataire")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let algorithm_used = json_data.get("AlgoSolrDestinataire")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let ready_for_routing = coordinates.0 != 0.0 || coordinates.1 != 0.0;
        let needs_validation = geocoding_quality != "Bon" || delivery_details.is_empty();
        let delivery_instructions = generate_delivery_instructions(&delivery_details, &customer);

        self.domicile_data = Some(OptimizedDomicileData {
            customer,
            original_address,
            geocoded_address,
            delivery_details,
            coordinates,
            barcode,
            postal_code,
            city,
            geocoding_quality,
            algorithm_used,
            ready_for_routing,
            needs_validation,
            delivery_instructions,
        });

        Ok(())
    }
}

// Función optimizada para extraer detalles de entrega
fn extract_delivery_details_optimized(original: &str, geocoded: &str) -> DeliveryDetails {
    let mut details = DeliveryDetails::new();

    if original.len() <= geocoded.len() {
        return details;
    }

    let original_upper = original.to_uppercase();
    let geocoded_upper = geocoded.to_uppercase();

    // Patrones específicos con extracción mejorada
    let patterns = vec![
        ("PORTE", &mut details.porte),
        ("APT", &mut details.apt),
        ("ETAGE", &mut details.etage),
        ("ESCALIER", &mut details.escalier),
        ("BATIMENT", &mut details.batiment),
        ("RESIDENCE", &mut details.residence),
        ("IMMEUBLE", &mut details.immeuble),
        ("TOUR", &mut details.tour),
        ("BLOC", &mut details.bloc),
        ("COULOIR", &mut details.couloir),
        ("COUR", &mut details.cour),
        ("JARDIN", &mut details.jardin),
    ];

    for (pattern, field) in patterns {
        if let Some(pos) = original_upper.find(pattern) {
            if !geocoded_upper.contains(&original_upper[pos..pos + pattern.len()]) {
                let remaining = &original[pos..];
                if let Some(space_pos) = remaining.find(' ') {
                    *field = Some(remaining[..space_pos].to_string());
                } else {
                    *field = Some(remaining.to_string());
                }
            }
        }
    }

    // Buscar otros patrones no categorizados
    let other_patterns = vec!["N°", "NUMERO", "NO", "FLOOR", "DOOR", "GATE"];
    for pattern in other_patterns {
        if let Some(pos) = original_upper.find(pattern) {
            if !geocoded_upper.contains(&original_upper[pos..pos + pattern.len()]) {
                let remaining = &original[pos..];
                if let Some(space_pos) = remaining.find(' ') {
                    details.other.push(remaining[..space_pos].to_string());
                } else {
                    details.other.push(remaining.to_string());
                }
            }
        }
    }

    details
}

// Función optimizada para extraer contacto del cliente
fn extract_customer_contact_optimized(json_data: &serde_json::Value) -> OptimizedCustomerContact {
    let name = json_data.get("nomDestinataire")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let phone = json_data.get("telephoneMobileDestinataire")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let phone_fixe = json_data.get("telephoneFixeDestinataire")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let email = json_data.get("emailDestinataire")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let email_valid = if let Some(email) = &email {
        !email.contains("aliexpress") && 
        !email.contains("temu") && 
        !email.contains("tiktok") && 
        !email.contains("amazon") && 
        !email.contains("ebay") &&
        !email.contains("google_") &&
        email.contains("@")
    } else {
        false
    };

    let can_receive_sms = !phone.is_empty();
    let can_receive_email = email_valid;
    let can_receive_whatsapp = !phone.is_empty();

    OptimizedCustomerContact {
        name,
        phone,
        phone_fixe,
        email,
        email_valid,
        can_receive_sms,
        can_receive_email,
        can_receive_whatsapp,
    }
}

// Función para generar instrucciones de entrega
fn generate_delivery_instructions(details: &DeliveryDetails, customer: &OptimizedCustomerContact) -> String {
    let mut instructions = Vec::new();

    if !details.is_empty() {
        instructions.push(format!("Detalles: {}", details.to_string()));
    }

    if customer.can_receive_sms {
        instructions.push("Cliente puede recibir SMS".to_string());
    }

    if customer.can_receive_whatsapp {
        instructions.push("Cliente puede recibir WhatsApp".to_string());
    }

    if customer.can_receive_email {
        instructions.push("Cliente puede recibir email".to_string());
    }

    if instructions.is_empty() {
        "Sin instrucciones especiales".to_string()
    } else {
        instructions.join(" | ")
    }
}

#[derive(Debug, Serialize)]
pub struct OptimizedExtractionReport {
    pub total_packages: usize,
    pub processing_time_ms: u64,
    pub relais_count: usize,
    pub rcs_count: usize,
    pub domicile_count: usize,
    pub packages_ready_for_routing: usize,
    pub packages_needing_validation: usize,
    pub customers_with_sms: usize,
    pub customers_with_email: usize,
    pub customers_with_whatsapp: usize,
    pub extraction_quality_distribution: HashMap<String, usize>,
    pub packages: Vec<OptimizedDeliveryPackage>,
}

impl OptimizedExtractionReport {
    pub fn new() -> Self {
        OptimizedExtractionReport {
            total_packages: 0,
            processing_time_ms: 0,
            relais_count: 0,
            rcs_count: 0,
            domicile_count: 0,
            packages_ready_for_routing: 0,
            packages_needing_validation: 0,
            customers_with_sms: 0,
            customers_with_email: 0,
            customers_with_whatsapp: 0,
            extraction_quality_distribution: HashMap::new(),
            packages: Vec::new(),
        }
    }

    pub fn add_package(&mut self, package: OptimizedDeliveryPackage) {
        self.total_packages += 1;
        self.processing_time_ms += package.processing_time_ms;

        match package.package_type.as_str() {
            "RELAIS" | "RELAISDIRECT" => {
                self.relais_count += 1;
                if let Some(relais) = &package.relais_data {
                    if relais.ready_for_routing {
                        self.packages_ready_for_routing += 1;
                    }
                }
            },
            "RCS" => {
                self.rcs_count += 1;
                if let Some(rcs) = &package.rcs_data {
                    if rcs.ready_for_routing {
                        self.packages_ready_for_routing += 1;
                    }
                }
            },
            "DOMICILE" => {
                self.domicile_count += 1;
                if let Some(domicile) = &package.domicile_data {
                    if domicile.ready_for_routing {
                        self.packages_ready_for_routing += 1;
                    }
                    if domicile.needs_validation {
                        self.packages_needing_validation += 1;
                    }
                    if domicile.customer.can_receive_sms {
                        self.customers_with_sms += 1;
                    }
                    if domicile.customer.can_receive_email {
                        self.customers_with_email += 1;
                    }
                    if domicile.customer.can_receive_whatsapp {
                        self.customers_with_whatsapp += 1;
                    }
                }
            },
            _ => {}
        }

        *self.extraction_quality_distribution.entry(package.extraction_quality.clone()).or_insert(0) += 1;
        self.packages.push(package);
    }

    pub fn generate_report(&self) {
        println!("\n{}", "=".repeat(70));
        println!("EXTRACCIÓN OPTIMIZADA DE DATOS - DELIVERY ROUTING");
        println!("{}", "=".repeat(70));

        println!("\n📦 Total de paquetes: {}", self.total_packages);
        println!("⏱️  Tiempo total de procesamiento: {} ms", self.processing_time_ms);
        println!("⚡ Tiempo promedio por paquete: {:.2} ms", 
                self.processing_time_ms as f64 / self.total_packages as f64);

        println!("\n🚚 Distribución por Tipo:");
        println!("  🏪 Point Relais: {} ({:.1}%)",
                self.relais_count,
                (self.relais_count as f64 / self.total_packages as f64) * 100.0);
        println!("  🏢 RCS: {} ({:.1}%)",
                self.rcs_count,
                (self.rcs_count as f64 / self.total_packages as f64) * 100.0);
        println!("  🏠 Domicilio: {} ({:.1}%)",
                self.domicile_count,
                (self.domicile_count as f64 / self.total_packages as f64) * 100.0);

        println!("\n✅ Estado de Procesamiento:");
        println!("  🚀 Listos para routing: {} ({:.1}%)",
                self.packages_ready_for_routing,
                (self.packages_ready_for_routing as f64 / self.total_packages as f64) * 100.0);
        println!("  ⚠️  Necesitan validación: {} ({:.1}%)",
                self.packages_needing_validation,
                (self.packages_needing_validation as f64 / self.total_packages as f64) * 100.0);

        println!("\n📱 Capacidades de Notificación:");
        println!("  📱 SMS: {} clientes ({:.1}%)",
                self.customers_with_sms,
                (self.customers_with_sms as f64 / self.domicile_count as f64) * 100.0);
        println!("  📧 Email: {} clientes ({:.1}%)",
                self.customers_with_email,
                (self.customers_with_email as f64 / self.domicile_count as f64) * 100.0);
        println!("  💬 WhatsApp: {} clientes ({:.1}%)",
                self.customers_with_whatsapp,
                (self.customers_with_whatsapp as f64 / self.domicile_count as f64) * 100.0);

        println!("\n🏆 Calidad de Extracción:");
        for (quality, count) in &self.extraction_quality_distribution {
            println!("  {}: {} ({:.1}%)", quality, count, (*count as f64 / self.total_packages as f64) * 100.0);
        }

        // Mostrar ejemplos optimizados
        self.show_optimized_examples();
    }

    fn show_optimized_examples(&self) {
        println!("\n🔍 Ejemplos Optimizados:");

        // Ejemplo RELAIS optimizado
        if let Some(relais) = self.packages.iter().find(|p| p.package_type == "RELAIS") {
            if let Some(data) = &relais.relais_data {
                println!("\n  🏪 Point Relais Optimizado:");
                println!("    Nombre: {}", data.name);
                println!("    Dirección: {}", data.address);
                println!("    Coordenadas: ({:.5}, {:.5})", data.coordinates.0, data.coordinates.1);
                println!("    Listo para routing: {}", data.ready_for_routing);
                println!("    Calidad: {}", data.geocoding_quality);
            }
        }

        // Ejemplo RCS optimizado
        if let Some(rcs) = self.packages.iter().find(|p| p.package_type == "RCS") {
            if let Some(data) = &rcs.rcs_data {
                println!("\n  🏢 RCS Optimizado:");
                println!("    Empresa: {}", data.company);
                println!("    Dirección: {}", data.address);
                println!("    Coordenadas: ({:.5}, {:.5})", data.coordinates.0, data.coordinates.1);
                println!("    Es prioritario: {}", data.is_priority);
                println!("    Listo para routing: {}", data.ready_for_routing);
            }
        }

        // Ejemplo DOMICILE optimizado
        if let Some(domicile) = self.packages.iter().find(|p| p.package_type == "DOMICILE") {
            if let Some(data) = &domicile.domicile_data {
                println!("\n  🏠 Domicilio Optimizado:");
                println!("    Cliente: {}", data.customer.name);
                println!("    Dirección original: {}", data.original_address);
                println!("    Dirección geocodificada: {}", data.geocoded_address);
                println!("    Detalles: {}", data.delivery_details.to_string());
                println!("    Instrucciones: {}", data.delivery_instructions);
                println!("    Necesita validación: {}", data.needs_validation);
                println!("    SMS: {} | Email: {} | WhatsApp: {}", 
                        data.customer.can_receive_sms,
                        data.customer.can_receive_email,
                        data.customer.can_receive_whatsapp);
            }
        }
    }
}
