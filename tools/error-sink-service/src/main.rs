use actix_web::middleware::{DefaultHeaders, Logger};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::Utc;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_REPORTS_DIR: &str = "reports";
const MINA_PUBLIC_KEY_LENGTH: usize = 44;

// List of valid error report categories
const VALID_CATEGORIES: [&str; 1] = ["blockProofFailure"];

#[derive(Clone)]
struct ServerConfig {
    port: u16,
    reports_dir: PathBuf,
    verify_signatures: bool,
    valid_categories: HashSet<String>,
}

/// Represents the JSON structure for an error report submission
#[derive(Deserialize, Serialize)]
struct ErrorReport {
    submitter: String, // Base58 encoded Mina public key
    category: String,
    data: String,      // Base64 encoded binary data
    signature: String, // Base64 encoded cryptographic signature
}

/// Handles POST requests to the /error-report endpoint
async fn handle_error_report(
    payload: web::Json<ErrorReport>,
    data: web::Data<ServerConfig>,
) -> Result<HttpResponse> {
    if !validate_base58_pubkey(&payload.submitter) {
        error!("Invalid base58 public key format: {}", payload.submitter);
        return Err(actix_web::error::ErrorBadRequest(
            "Invalid submitter public key format",
        ));
    }

    if !data.valid_categories.contains(&payload.category) {
        error!("Invalid error category: {}", payload.category);
        return Err(actix_web::error::ErrorBadRequest("Invalid error category."));
    }

    let now = Utc::now();
    let uuid = Uuid::new_v4();
    let timestamp = now.format("%Y%m%d-%H%M%S").to_string();

    let reports_dir = &data.reports_dir;

    if !reports_dir.exists() {
        fs::create_dir_all(reports_dir).map_err(|e| {
            error!("Failed to create reports directory: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to create reports directory")
        })?;
    }

    // Use the category directly since it's already validated
    let file_name = format!(
        "{}/{}-{}_{}_{}.report",
        reports_dir.display(),
        payload.category, // No need to sanitize - it's from our valid list
        payload.submitter,
        timestamp,
        uuid
    );

    let data_bytes = BASE64.decode(&payload.data).map_err(|e| {
        error!("Failed to decode base64 data: {}", e);
        actix_web::error::ErrorBadRequest("Invalid base64 data")
    })?;

    if data.verify_signatures {
        let _sig_bytes = match BASE64.decode(&payload.signature) {
            Ok(bytes) => bytes,
            Err(e) => {
                warn!("Invalid signature format: {}", e);
                return Err(actix_web::error::ErrorBadRequest(
                    "Invalid signature format",
                ));
            }
        };

        // TODO: verify signature here
        info!("Signature verification would occur here (not implemented yet)");
    }

    let mut file = File::create(&file_name).map_err(|e| {
        error!("Failed to create report file: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to create report file")
    })?;

    file.write_all(&data_bytes).map_err(|e| {
        error!("Failed to write to report file: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to write data to file")
    })?;

    info!("Saved error report to: {}", file_name);

    Ok(HttpResponse::Created()
        .append_header(("Location", file_name.clone()))
        .body(format!("Report saved as {}", file_name)))
}

/// Validate that a string is a proper base58 encoded Mina public key
fn validate_base58_pubkey(pubkey: &str) -> bool {
    if pubkey.len() != MINA_PUBLIC_KEY_LENGTH {
        return false;
    }

    bs58::decode(pubkey).into_vec().is_ok()
}

/// Structure to hold error report file information
struct ReportFileInfo {
    filename: String,
    submitter: String, // Extracted from filename
    category: String,  // Extracted from filename
    size: u64,
    timestamp: String,
}

/// Extract submitter and category from filename
fn extract_info_from_filename(filename: &str) -> (String, String) {
    let parts: Vec<&str> = filename.split('-').collect();

    if parts.len() >= 2 {
        let category = parts[0].to_string();

        // The submitter is now between the first '-' and the first '_'
        if parts.len() > 1 {
            let remaining = parts[1..].join("-"); // Rejoin in case category had hyphens
            let submitter_parts: Vec<&str> = remaining.split('_').collect();

            if !submitter_parts.is_empty() {
                return (category, submitter_parts[0].to_string());
            }
        }

        return (category, "Unknown".to_string());
    }

    ("Unknown".to_string(), "Unknown".to_string())
}

/// Retrieves and renders a list of all error reports
async fn index(data: web::Data<ServerConfig>) -> Result<HttpResponse> {
    let entries = fs::read_dir(&data.reports_dir).map_err(|e| {
        error!("Failed to read reports directory: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to read reports directory")
    })?;

    let mut files = Vec::new();
    for entry in entries.flatten() {
        if let Some(file_name) = entry.file_name().to_str() {
            if file_name.ends_with(".report") {
                let metadata = entry.metadata().ok();
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified = metadata
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        chrono::DateTime::<chrono::Utc>::from(t)
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string()
                    })
                    .unwrap_or_else(|| "Unknown".to_string());

                let (category, submitter) = extract_info_from_filename(file_name);

                files.push(ReportFileInfo {
                    filename: file_name.to_string(),
                    submitter,
                    category,
                    size,
                    timestamp: modified,
                });
            }
        }
    }

    // Sort by timestamp (newest first)
    files.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let mut html = String::from(
        "<!DOCTYPE html>
<html>
<head>
    <title>Error Reports Index</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        h1 { color: #333; }
        table { border-collapse: collapse; width: 100%; }
        th, td { text-align: left; padding: 8px; }
        tr:nth-child(even) { background-color: #f2f2f2; }
        th { background-color: #4CAF50; color: white; }
        a { color: #0066cc; }
        .category { font-weight: bold; }
    </style>
</head>
<body>
    <h1>Error Reports Index</h1>
    <table>
        <tr>
            <th>Category</th>
            <th>Submitter</th>
            <th>File Name</th>
            <th>Size</th>
            <th>Date</th>
            <th>Actions</th>
        </tr>",
    );

    for file in files {
        let size_str = if file.size < 1024 {
            format!("{} B", file.size)
        } else if file.size < 1024 * 1024 {
            format!("{:.2} KB", file.size as f64 / 1024.0)
        } else {
            format!("{:.2} MB", file.size as f64 / (1024.0 * 1024.0))
        };

        html.push_str(&format!(
            "
        <tr>
            <td class=\"category\">{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td><a href=\"/download/{}\">Download</a></td>
        </tr>",
            file.category, file.submitter, file.filename, size_str, file.timestamp, file.filename
        ));
    }

    html.push_str(
        "
    </table>
</body>
</html>",
    );

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

/// Handles download requests for specific report files
async fn download_file(req: HttpRequest, data: web::Data<ServerConfig>) -> Result<HttpResponse> {
    let file_name = req.match_info().get("filename").unwrap();
    let file_path = data.reports_dir.join(file_name);

    if !file_path.exists() || !file_path.is_file() || !file_path.starts_with(&data.reports_dir) {
        return Ok(HttpResponse::NotFound().body("File not found"));
    }

    let content = fs::read(&file_path).map_err(|e| {
        error!("Failed to read file {}: {}", file_path.display(), e);
        actix_web::error::ErrorInternalServerError("Failed to read file")
    })?;

    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .append_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", file_name),
        ))
        .body(content))
}

/// Initialize the server configuration from environment variables
fn init_config() -> ServerConfig {
    let port = std::env::var("ERROR_SINK_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);

    let reports_dir =
        std::env::var("ERROR_SINK_DIR").unwrap_or_else(|_| DEFAULT_REPORTS_DIR.to_string());

    // Option to disable signature verification (enabled by default)
    let verify_signatures = std::env::var("ERROR_SINK_VERIFY_SIGNATURES")
        .map(|v| v != "0" && v.to_lowercase() != "false")
        .unwrap_or(true);

    let valid_categories: HashSet<String> =
        VALID_CATEGORIES.iter().map(|&s| s.to_string()).collect();

    ServerConfig {
        port,
        reports_dir: PathBuf::from(reports_dir),
        verify_signatures,
        valid_categories,
    }
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let config = init_config();

    if !config.reports_dir.exists() {
        fs::create_dir_all(&config.reports_dir)?;
    }

    info!("Starting error sink service on port {}", config.port);
    info!("Storing error reports in: {}", config.reports_dir.display());
    info!("Valid error categories: {:?}", config.valid_categories);

    let config_data = web::Data::new(config.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(config_data.clone())
            .app_data(
                web::JsonConfig::default()
                    .limit(100 * 1024 * 1024) // 100MB limit for JSON payload
                    .error_handler(|err, _| {
                        error!("JSON parsing error: {}", err);
                        actix_web::error::ErrorBadRequest(err)
                    }),
            )
            .wrap(DefaultHeaders::new().add(("X-Error-Sink", "OpenMina")))
            .wrap(Logger::default())
            .service(web::resource("/error-report").route(web::post().to(handle_error_report)))
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/download/{filename}").route(web::get().to(download_file)))
            .default_service(web::route().to(HttpResponse::NotFound))
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await?;

    Ok(())
}
