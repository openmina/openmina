use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::io::{self, Read};

#[derive(Debug, clap::Args)]
pub struct Run {
    /// GraphQL query to execute. If not provided, reads from stdin.
    pub query: Option<String>,

    /// GraphQL variables as JSON string
    #[arg(long, short = 'v')]
    pub variables: Option<String>,

    /// Read query from file
    #[arg(long, short = 'f')]
    pub file: Option<String>,

    /// GraphQL server URL
    #[arg(long, default_value = "http://localhost:3000/graphql")]
    pub node: String,
}

#[derive(Debug, Serialize)]
struct GraphQLRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct GraphQLResponse {
    data: Option<serde_json::Value>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
}

impl Run {
    pub fn run(self) -> Result<()> {
        // Get the query from various sources
        let query = if let Some(query) = self.query {
            query
        } else if let Some(file_path) = self.file {
            std::fs::read_to_string(&file_path)
                .map_err(|e| anyhow!("Failed to read file '{}': {}", file_path, e))?
        } else {
            // Read from stdin
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .map_err(|e| anyhow!("Failed to read from stdin: {}", e))?;
            buffer
        };

        // Parse variables if provided
        let variables = if let Some(vars_str) = self.variables {
            let vars: serde_json::Value = serde_json::from_str(&vars_str)
                .map_err(|e| anyhow!("Failed to parse variables JSON: {}", e))?;
            Some(vars)
        } else {
            None
        };

        // Build GraphQL request
        let request = GraphQLRequest { query, variables };

        // Execute the query
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&self.node)
            .json(&request)
            .send()
            .map_err(|e| anyhow!("Failed to connect to GraphQL server: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "GraphQL server returned error: {}",
                response.status()
            ));
        }

        let graphql_response: GraphQLResponse = response
            .json()
            .map_err(|e| anyhow!("Failed to parse GraphQL response: {}", e))?;

        // Display the response
        if let Some(errors) = &graphql_response.errors {
            if !errors.is_empty() {
                eprintln!("Errors:");
                for error in errors {
                    eprintln!("  - {}", error.message);
                }
                if graphql_response.data.is_none() {
                    return Err(anyhow!("GraphQL query failed with errors"));
                }
            }
        }

        if let Some(data) = graphql_response.data {
            let formatted = serde_json::to_string_pretty(&data)
                .unwrap_or_else(|_| serde_json::to_string(&data).unwrap_or_default());
            println!("{}", formatted);
        }

        Ok(())
    }
}
