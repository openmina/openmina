use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, clap::Args)]
pub struct List {
    /// GraphQL server URL
    #[arg(long, default_value = "http://localhost:3000/graphql")]
    pub node: String,
}

#[derive(Debug, Serialize)]
struct GraphQLRequest {
    query: String,
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

impl List {
    pub fn run(self) -> Result<()> {
        // GraphQL introspection query to get all queries, mutations, and subscriptions
        let introspection_query = r#"
            query IntrospectSchema {
                __schema {
                    queryType { name }
                    mutationType { name }
                    subscriptionType { name }
                    types {
                        name
                        kind
                        fields {
                            name
                        }
                    }
                }
            }
        "#;

        let request = GraphQLRequest {
            query: introspection_query.to_string(),
        };

        // Make the introspection request
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

        if let Some(errors) = graphql_response.errors {
            for error in errors {
                eprintln!("GraphQL Error: {}", error.message);
            }
            return Err(anyhow!("GraphQL introspection failed"));
        }

        let data = graphql_response
            .data
            .ok_or_else(|| anyhow!("No data in GraphQL response"))?;

        // Parse and display the schema
        self.display_endpoints(&data)?;

        Ok(())
    }

    fn display_endpoints(&self, schema_data: &serde_json::Value) -> Result<()> {
        let schema = schema_data
            .get("__schema")
            .ok_or_else(|| anyhow!("Invalid schema response"))?;

        let types = schema
            .get("types")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("Invalid types in schema"))?;

        // Helper function to get and display endpoints
        let get_endpoints = |type_name: &str| -> Vec<String> {
            types
                .iter()
                .find(|t| {
                    t.get("name")
                        .and_then(|n| n.as_str())
                        .map(|n| n == type_name)
                        .unwrap_or(false)
                })
                .and_then(|t| t.get("fields"))
                .and_then(|f| f.as_array())
                .map(|fields| {
                    let mut names: Vec<String> = fields
                        .iter()
                        .filter_map(|f| f.get("name").and_then(|n| n.as_str()).map(String::from))
                        .collect();
                    names.sort();
                    names
                })
                .unwrap_or_default()
        };

        // Get queries
        let queries = if let Some(query_type_name) = schema
            .get("queryType")
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
        {
            get_endpoints(query_type_name)
        } else {
            Vec::new()
        };

        // Get mutations
        let mutations = if let Some(mutation_type_name) = schema
            .get("mutationType")
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
        {
            get_endpoints(mutation_type_name)
        } else {
            Vec::new()
        };

        // Get subscriptions
        let subscriptions = if let Some(subscription_type_name) = schema
            .get("subscriptionType")
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
        {
            get_endpoints(subscription_type_name)
        } else {
            Vec::new()
        };

        // Display queries
        if !queries.is_empty() {
            println!("Queries ({}):", queries.len());
            for query in &queries {
                println!("  - {}", query);
            }
        }

        // Display mutations
        if !mutations.is_empty() {
            println!("Mutations ({}):", mutations.len());
            for mutation in &mutations {
                println!("  - {}", mutation);
            }
        }

        // Display subscriptions
        if !subscriptions.is_empty() {
            println!("Subscriptions ({}):", subscriptions.len());
            for subscription in &subscriptions {
                println!("  - {}", subscription);
            }
        }

        // Display example curl command
        let node_url = &self.node;
        println!(
            "\nExample: curl -s -X POST {} -H 'Content-Type: application/json' \\",
            node_url
        );
        println!("         -d '{{\"query\": \"{{ daemonStatus {{ chainId blockchainLength numAccounts }} }}\"}}'");

        Ok(())
    }
}
