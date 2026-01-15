use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, clap::Args)]
pub struct Inspect {
    /// The GraphQL endpoint to inspect (e.g., 'account', 'sync_status', 'send_payment')
    pub endpoint: String,

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

impl Inspect {
    pub fn run(self) -> Result<()> {
        // GraphQL introspection query to get field details
        let introspection_query = r#"
            query IntrospectSchema {
                __schema {
                    queryType { name }
                    mutationType { name }
                    types {
                        name
                        kind
                        description
                        fields {
                            name
                            description
                            args {
                                name
                                description
                                type {
                                    name
                                    kind
                                    ofType {
                                        name
                                        kind
                                        ofType {
                                            name
                                            kind
                                        }
                                    }
                                }
                                defaultValue
                            }
                            type {
                                name
                                kind
                                ofType {
                                    name
                                    kind
                                    ofType {
                                        name
                                        kind
                                    }
                                }
                            }
                        }
                        inputFields {
                            name
                            description
                            type {
                                name
                                kind
                                ofType {
                                    name
                                    kind
                                    ofType {
                                        name
                                        kind
                                    }
                                }
                            }
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

        // Parse the schema and find the requested endpoint
        let (_has_required_args, args) = self.display_endpoint_info(&data)?;

        // Show example with or without variables
        self.display_example_output(&args)?;

        Ok(())
    }

    fn display_endpoint_info(
        &self,
        schema_data: &serde_json::Value,
    ) -> Result<(bool, Vec<(String, bool)>)> {
        let schema = schema_data
            .get("__schema")
            .ok_or_else(|| anyhow!("Invalid schema response"))?;

        let types = schema
            .get("types")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("Invalid types in schema"))?;

        // Find Query or Mutation type
        let query_type_name = schema
            .get("queryType")
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str());

        let mutation_type_name = schema
            .get("mutationType")
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str());

        let mut found = false;
        let mut has_required_args = false;
        let mut args_info = Vec::new();

        // Search in Query type
        if let Some(query_name) = query_type_name {
            if let Some(query_type) = types.iter().find(|t| {
                t.get("name")
                    .and_then(|n| n.as_str())
                    .map(|n| n == query_name)
                    .unwrap_or(false)
            }) {
                if let Some(fields) = query_type.get("fields").and_then(|f| f.as_array()) {
                    if let Some(field) = fields.iter().find(|f| {
                        f.get("name")
                            .and_then(|n| n.as_str())
                            .map(|n| n == self.endpoint)
                            .unwrap_or(false)
                    }) {
                        println!("Endpoint: {} (Query)", self.endpoint);
                        let (has_req, args) = self.display_field(field)?;
                        has_required_args = has_req;
                        args_info = args;
                        found = true;
                    }
                }
            }
        }

        // Search in Mutation type if not found in Query
        if !found {
            if let Some(mutation_name) = mutation_type_name {
                if let Some(mutation_type) = types.iter().find(|t| {
                    t.get("name")
                        .and_then(|n| n.as_str())
                        .map(|n| n == mutation_name)
                        .unwrap_or(false)
                }) {
                    if let Some(fields) = mutation_type.get("fields").and_then(|f| f.as_array()) {
                        if let Some(field) = fields.iter().find(|f| {
                            f.get("name")
                                .and_then(|n| n.as_str())
                                .map(|n| n == self.endpoint)
                                .unwrap_or(false)
                        }) {
                            println!("Endpoint: {} (Mutation)", self.endpoint);
                            let (has_req, args) = self.display_field(field)?;
                            has_required_args = has_req;
                            args_info = args;
                            found = true;
                        }
                    }
                }
            }
        }

        if !found {
            return Err(anyhow!(
                "Endpoint '{}' not found. Use 'mina internal graphql list' to see available endpoints.",
                self.endpoint
            ));
        }

        Ok((has_required_args, args_info))
    }

    fn display_field(&self, field: &serde_json::Value) -> Result<(bool, Vec<(String, bool)>)> {
        println!();

        // Description
        if let Some(desc) = field.get("description").and_then(|d| d.as_str()) {
            println!("Description:");
            println!("  {}", desc);
            println!();
        }

        // Arguments
        let mut has_required_args = false;
        let mut args_info = Vec::new();
        if let Some(args) = field.get("args").and_then(|a| a.as_array()) {
            if !args.is_empty() {
                println!("Arguments:");
                for arg in args {
                    let name = arg.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    let is_required = self.is_required_type(arg.get("type"));
                    let desc = arg
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("");

                    if is_required {
                        has_required_args = true;
                    }

                    args_info.push((name.to_string(), is_required));

                    let requirement = if is_required {
                        " (required)"
                    } else {
                        " (optional)"
                    };
                    print!("  {}{}", name, requirement);
                    if !desc.is_empty() {
                        print!(" - {}", desc);
                    }
                    println!();
                }
                println!();
            }
        }

        Ok((has_required_args, args_info))
    }

    fn is_required_type(&self, type_val: Option<&serde_json::Value>) -> bool {
        if let Some(type_val) = type_val {
            if let Some(kind) = type_val.get("kind").and_then(|k| k.as_str()) {
                return kind == "NON_NULL";
            }
        }
        false
    }

    fn display_example_output(&self, args: &[(String, bool)]) -> Result<()> {
        if args.is_empty() {
            // Simple query without arguments
            println!("\nExample Query:");
            println!("  query {{");
            println!("    {}", self.endpoint);
            println!("  }}");
            println!();

            // Show curl command
            let query_escaped = format!("query {{ {} }}", self.endpoint).replace('"', "\\\"");
            println!("Curl Command:");
            println!("  curl -X POST {} \\", self.node);
            println!("    -H \"Content-Type: application/json\" \\");
            println!("    -d '{{\"query\": \"{}\"}}'", query_escaped);
            println!();

            // Show CLI run command
            println!("CLI Command:");
            println!(
                "  mina internal graphql run 'query {{ {} }}' \\",
                self.endpoint
            );
            println!("    --node {}", self.node);
            println!();

            // Execute the example query
            let query = format!("query {{ {} }}", self.endpoint);
            self.execute_and_display_query(&query)?;
        } else {
            // Query with arguments - show example with variables
            println!("\nExample Query with Variables:");
            let arg_list: Vec<String> = args
                .iter()
                .map(|(name, is_req)| {
                    let type_suffix = if *is_req { "!" } else { "" };
                    format!("${}: Type{}", name, type_suffix)
                })
                .collect();
            let arg_usage: Vec<String> = args
                .iter()
                .map(|(name, _)| format!("{}: ${}", name, name))
                .collect();

            println!("  query({}) {{", arg_list.join(", "));
            println!("    {}({})", self.endpoint, arg_usage.join(", "));
            println!("  }}");
            println!();

            // Generate example variable values
            let example_vars: Vec<String> = args
                .iter()
                .map(|(name, _)| {
                    // Provide sensible default values based on common arg names
                    let value = match name.as_str() {
                        "maxLength" | "max" | "limit" => "10",
                        "offset" | "skip" => "0",
                        _ => "\"example_value\"",
                    };
                    format!("\"{}\": {}", name, value)
                })
                .collect();

            println!("Example Variables:");
            println!("  {{");
            for (i, var) in example_vars.iter().enumerate() {
                if i < example_vars.len() - 1 {
                    println!("    {},", var);
                } else {
                    println!("    {}", var);
                }
            }
            println!("  }}");
            println!();

            // Show CLI run command with variables
            println!("CLI Command:");
            let query_with_vars = format!(
                "query({}) {{ {}({}) }}",
                arg_list.join(", "),
                self.endpoint,
                arg_usage.join(", ")
            );
            println!("  mina internal graphql run \\");
            println!("    '{}' \\", query_with_vars);
            println!("    -v '{{{}}}' \\", example_vars.join(", "));
            println!("    --node {}", self.node);
            println!();

            println!(
                "Note: Adjust the variable values and types according to the endpoint's schema."
            );
            println!("      Use 'mina internal graphql run --help' for more information.");
            println!();
        }

        Ok(())
    }

    fn execute_and_display_query(&self, query: &str) -> Result<()> {
        let request = GraphQLRequest {
            query: query.to_string(),
        };

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&self.node)
            .json(&request)
            .send()
            .map_err(|e| anyhow!("Failed to execute example query: {}", e))?;

        if !response.status().is_success() {
            println!(
                "Warning: Could not fetch example output (status: {})",
                response.status()
            );
            return Ok(());
        }

        let graphql_response: GraphQLResponse = response
            .json()
            .map_err(|e| anyhow!("Failed to parse example response: {}", e))?;

        println!("Example Response:");
        if let Some(data) = graphql_response.data {
            let formatted = serde_json::to_string_pretty(&data)
                .unwrap_or_else(|_| serde_json::to_string(&data).unwrap_or_default());
            println!("{}", formatted);
        }

        if let Some(errors) = graphql_response.errors {
            if !errors.is_empty() {
                println!("\nErrors:");
                for error in errors {
                    println!("  - {}", error.message);
                }
            }
        }

        Ok(())
    }
}
