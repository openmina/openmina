use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(cmd) = std::env::args().nth(1) else {
        return print_usage();
    };

    match cmd.as_str() {
        #[cfg(feature = "openapi-gen")]
        "openapi" => {
            if let Err(e) = generate_openapi() {
                eprintln!("Error: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        _ => print_usage(),
    }
}

fn print_usage() -> ExitCode {
    eprintln!("Usage: cargo xtask <command>");
    eprintln!();
    eprintln!("Commands:");
    #[cfg(feature = "openapi-gen")]
    eprintln!("  openapi [output]  Generate OpenAPI spec");
    #[cfg(not(feature = "openapi-gen"))]
    eprintln!("  (no commands available - enable features)");
    ExitCode::FAILURE
}

#[cfg(feature = "openapi-gen")]
fn generate_openapi() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args().nth(2);
    let spec = mina_node_native::http_server::openapi_spec();
    let json = serde_json::to_string_pretty(&spec)?;
    match output {
        Some(path) => {
            std::fs::write(&path, &json)?;
            eprintln!("OpenAPI spec written to {path}");
        }
        None => print!("{json}"),
    }
    Ok(())
}
