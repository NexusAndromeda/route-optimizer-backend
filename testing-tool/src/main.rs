use colored::*;
use serde_json::json;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "🚚 Colis Privé Testing Tool".bright_blue().bold());
    println!("{}", "=====================================".bright_blue());
    println!();

    // Paso 1: Pedir credenciales
    let credentials = get_credentials()?;
    
    // Paso 2: Autenticarse y obtener token
    let token = authenticate(&credentials).await?;
    
    // Paso 3: Menú principal
    loop {
        println!();
        println!("{}", "📋 MENÚ PRINCIPAL".bright_green().bold());
        println!("{}", "==================".bright_green());
        println!("1. 🔍 Probar obtener paquetes");
        println!("2. 🚪 Salir");
        print!("{}", "Selecciona una opción (1-2): ".bright_yellow());
        io::stdout().flush()?;
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();
        
        match choice {
            "1" => {
                test_get_packages(&credentials, &token).await?;
            }
            "2" => {
                println!("{}", "👋 ¡Hasta luego!".bright_green());
                break;
            }
            _ => {
                println!("{}", "❌ Opción inválida. Intenta de nuevo.".bright_red());
            }
        }
    }
    
    Ok(())
}

fn get_credentials() -> Result<Credentials, Box<dyn std::error::Error>> {
    println!("{}", "🔐 CREDENCIALES DE COLIS PRIVÉ".bright_cyan().bold());
    println!("{}", "===============================".bright_cyan());
    
    print!("{}", "Username: ".bright_yellow());
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();
    
    print!("{}", "Password: ".bright_yellow());
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim().to_string();
    
    print!("{}", "Matrícula: ".bright_yellow());
    io::stdout().flush()?;
    let mut matricule = String::new();
    io::stdin().read_line(&mut matricule)?;
    let matricule = matricule.trim().to_string();
    
    print!("{}", "Sociedad (ej: PCP0010699): ".bright_yellow());
    io::stdout().flush()?;
    let mut societe = String::new();
    io::stdin().read_line(&mut societe)?;
    let societe = societe.trim().to_string();
    
    Ok(Credentials {
        username,
        password,
        matricule,
        societe,
    })
}

async fn authenticate(credentials: &Credentials) -> Result<String, Box<dyn std::error::Error>> {
    println!();
    println!("{}", "🔐 AUTENTICANDO CON COLIS PRIVÉ...".bright_cyan().bold());
    println!("{}", "===================================".bright_cyan());
    
    let auth_url = "https://wsauthentificationexterne.colisprive.com/api/auth/login/Membership";
    let login_field = format!("{}_{}", credentials.societe, credentials.username.trim());
    
    let payload = json!({
        "login": login_field,
        "password": credentials.password,
        "societe": credentials.societe,
        "commun": {
            "dureeTokenInHour": 24
        }
    });
    
    println!("{}", "📤 URL:".bright_blue());
    println!("{}", auth_url);
    println!();
    
    println!("{}", "📦 Payload:".bright_blue());
    println!("{}", serde_json::to_string_pretty(&payload)?);
    println!();
    
    // Ejecutar curl
    let curl_output = std::process::Command::new("curl")
        .arg("-X")
        .arg("POST")
        .arg(auth_url)
        .arg("-H")
        .arg("Accept: application/json, text/plain, */*")
        .arg("-H")
        .arg("Accept-Language: fr-FR,fr;q=0.6")
        .arg("-H")
        .arg("Connection: keep-alive")
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-H")
        .arg("Origin: https://gestiontournee.colisprive.com")
        .arg("-H")
        .arg("Referer: https://gestiontournee.colisprive.com/")
        .arg("-H")
        .arg("Sec-Fetch-Dest: empty")
        .arg("-H")
        .arg("Sec-Fetch-Mode: cors")
        .arg("-H")
        .arg("Sec-Fetch-Site: same-site")
        .arg("-H")
        .arg("Sec-GPC: 1")
        .arg("-H")
        .arg("User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36")
        .arg("-H")
        .arg("sec-ch-ua: \"Chromium\";v=\"140\", \"Not=A?Brand\";v=\"24\", \"Brave\";v=\"140\"")
        .arg("-H")
        .arg("sec-ch-ua-mobile: ?0")
        .arg("-H")
        .arg("sec-ch-ua-platform: \"macOS\"")
        .arg("--data-raw")
        .arg(&serde_json::to_string(&payload)?)
        .arg("--max-time")
        .arg("30")
        .arg("--verbose")
        .arg("--show-error")
        .output()?;
    
    println!("{}", "📥 RESPUESTA COMPLETA:".bright_green().bold());
    println!("{}", "=====================".bright_green());
    
    // Mostrar stderr (headers y info de curl)
    if !curl_output.stderr.is_empty() {
        println!("{}", "🔍 Headers y Info de cURL:".bright_blue());
        println!("{}", String::from_utf8_lossy(&curl_output.stderr));
        println!();
    }
    
    // Mostrar stdout (body de la respuesta)
    if !curl_output.stdout.is_empty() {
        println!("{}", "📄 Body de la Respuesta:".bright_blue());
        let response_body = String::from_utf8_lossy(&curl_output.stdout);
        println!("{}", response_body);
        println!();
        
        // Intentar parsear el JSON y extraer el token
        if let Ok(json_response) = serde_json::from_str::<serde_json::Value>(&response_body) {
            if let Some(tokens) = json_response.get("tokens") {
                if let Some(sso_hopps) = tokens.get("SsoHopps") {
                    if let Some(token_str) = sso_hopps.as_str() {
                        println!("{}", "✅ TOKEN EXTRAÍDO:".bright_green().bold());
                        println!("{}", token_str);
                        println!();
                        return Ok(token_str.to_string());
                    }
                }
            }
            
            // Si no está en tokens, buscar en habilitationAD
            if let Some(hab_ad) = json_response.get("habilitationAD") {
                if let Some(sso_hopps) = hab_ad.get("SsoHopps") {
                    if let Some(sso_array) = sso_hopps.as_array() {
                        if let Some(first_item) = sso_array.get(0) {
                            if let Some(valeur) = first_item.get("valeur") {
                                if let Some(token_str) = valeur.as_str() {
                                    println!("{}", "✅ TOKEN EXTRAÍDO (desde habilitationAD):".bright_green().bold());
                                    println!("{}", token_str);
                                    println!();
                                    return Ok(token_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Err("❌ No se pudo extraer el token de la respuesta".into())
}

async fn test_get_packages(credentials: &Credentials, token: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("{}", "📦 PROBANDO OBTENER PAQUETES...".bright_cyan().bold());
    println!("{}", "=================================".bright_cyan());
    
    let tournee_url = "https://wstournee-v2.colisprive.com/WS-TourneeColis/api/getTourneeByMatriculeDistributeurDateDebut_POST";
    let matricule_completo = format!("{}_{}", credentials.societe, credentials.matricule.trim());
    
    let payload = json!({
        "DateDebut": "2025-09-11",
        "Matricule": matricule_completo
    });
    
    println!("{}", "📤 URL:".bright_blue());
    println!("{}", tournee_url);
    println!();
    
    println!("{}", "📦 Payload:".bright_blue());
    println!("{}", serde_json::to_string_pretty(&payload)?);
    println!();
    
    // Ejecutar curl
    let curl_output = std::process::Command::new("curl")
        .arg("-X")
        .arg("POST")
        .arg(tournee_url)
        .arg("-H")
        .arg("Accept: application/json, text/plain, */*")
        .arg("-H")
        .arg("Accept-Language: fr-FR,fr;q=0.6")
        .arg("-H")
        .arg("Connection: keep-alive")
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-H")
        .arg("Origin: https://gestiontournee.colisprive.com")
        .arg("-H")
        .arg("Referer: https://gestiontournee.colisprive.com/")
        .arg("-H")
        .arg("Sec-Fetch-Dest: empty")
        .arg("-H")
        .arg("Sec-Fetch-Mode: cors")
        .arg("-H")
        .arg("Sec-Fetch-Site: same-site")
        .arg("-H")
        .arg("Sec-GPC: 1")
        .arg("-H")
        .arg(format!("SsoHopps: {}", token))
        .arg("-H")
        .arg("User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36")
        .arg("-H")
        .arg("sec-ch-ua: \"Chromium\";v=\"140\", \"Not=A?Brand\";v=\"24\", \"Brave\";v=\"140\"")
        .arg("-H")
        .arg("sec-ch-ua-mobile: ?0")
        .arg("-H")
        .arg("sec-ch-ua-platform: \"macOS\"")
        .arg("--data-raw")
        .arg(&serde_json::to_string(&payload)?)
        .arg("--max-time")
        .arg("30")
        .arg("--verbose")
        .arg("--show-error")
        .output()?;
    
    println!("{}", "📥 RESPUESTA COMPLETA:".bright_green().bold());
    println!("{}", "=====================".bright_green());
    
    // Mostrar stderr (headers y info de curl)
    if !curl_output.stderr.is_empty() {
        println!("{}", "🔍 Headers y Info de cURL:".bright_blue());
        println!("{}", String::from_utf8_lossy(&curl_output.stderr));
        println!();
    }
    
    // Mostrar stdout (body de la respuesta)
    if !curl_output.stdout.is_empty() {
        println!("{}", "📄 Body de la Respuesta:".bright_blue());
        let response_body = String::from_utf8_lossy(&curl_output.stdout);
        println!("{}", response_body);
        println!();
        
        // Intentar parsear el JSON y mostrar información útil
        if let Ok(json_response) = serde_json::from_str::<serde_json::Value>(&response_body) {
            if let Some(infos_tournee) = json_response.get("InfosTournee") {
                println!("{}", "🏁 INFORMACIÓN DE TOURNÉE:".bright_green().bold());
                println!("{}", serde_json::to_string_pretty(infos_tournee)?);
                println!();
            }
            
            if let Some(lst_lieu_article) = json_response.get("LstLieuArticle") {
                if let Some(arr) = lst_lieu_article.as_array() {
                    println!("{}", format!("📦 PAQUETES ENCONTRADOS: {} elementos", arr.len()).bright_green().bold());
                    if !arr.is_empty() {
                        println!("{}", serde_json::to_string_pretty(lst_lieu_article)?);
                    } else {
                        println!("{}", "⚠️ No hay paquetes en LstLieuArticle".bright_yellow());
                    }
                }
            }
        }
    }
    
    Ok(())
}

#[derive(Debug)]
struct Credentials {
    username: String,
    password: String,
    matricule: String,
    societe: String,
}