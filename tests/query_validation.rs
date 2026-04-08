//! Validates all raw GraphQL query strings in the codebase against the schema.
//!
//! This test extracts every GraphQL query/mutation embedded in source files and:
//! 1. Parses each one to verify it is syntactically valid GraphQL
//! 2. Validates that top-level fields exist in the schema's Query/Mutation types
//! 3. Validates that variable types reference real schema types

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// A query extracted from a source file, with location info for error reporting.
#[derive(Debug)]
struct ExtractedQuery {
    file: PathBuf,
    line: usize,
    query: String,
}

/// Extract all GraphQL query strings from Rust source files.
///
/// Handles two patterns:
/// - Raw string literals: r#"query { ... }"# (single and multi-line)
/// - Regular string literals: "query { ... }" (single-line, starting with query/mutation)
fn extract_queries_from_file(path: &Path) -> Vec<ExtractedQuery> {
    let content = std::fs::read_to_string(path).expect("Failed to read source file");
    let mut queries = Vec::new();

    // Extract r#"..."# raw string literals that contain GraphQL
    let mut search_from = 0;
    while let Some(start) = content[search_from..].find("r#\"") {
        let abs_start = search_from + start;
        let content_start = abs_start + 3; // skip r#"

        if let Some(end_offset) = content[content_start..].find("\"#") {
            let abs_end = content_start + end_offset;
            let raw_str = &content[content_start..abs_end];

            // Only include strings that look like GraphQL operations
            let trimmed = raw_str.trim();
            if trimmed.starts_with("query")
                || trimmed.starts_with("mutation")
                || trimmed.starts_with("subscription")
                || trimmed.starts_with("{")
            {
                let line = content[..abs_start].matches('\n').count() + 1;
                queries.push(ExtractedQuery {
                    file: path.to_path_buf(),
                    line,
                    query: raw_str.to_string(),
                });
            }

            search_from = abs_end + 2; // skip past "#
        } else {
            break;
        }
    }

    // Extract regular "..." strings passed to query_raw that look like GraphQL.
    // These are single-line strings like: "query { teams { nodes { id } } }"
    // We look for patterns like query_raw("query..." or query_raw(\n"query..."
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip lines that are inside raw strings (already handled above)
        // or that are comments
        if trimmed.starts_with("//") || trimmed.starts_with("r#") {
            continue;
        }

        // Match regular string literals that look like GraphQL queries.
        // These appear as: "query { ... }" or "query($var: Type!) { ... }"
        // Find all occurrences of quoted strings starting with query/mutation on this line
        let mut pos = 0;
        while pos < trimmed.len() {
            if let Some(q_start) = trimmed[pos..].find('"') {
                let abs_q_start = pos + q_start + 1;
                if abs_q_start >= trimmed.len() {
                    break;
                }
                // Find closing quote (not escaped)
                if let Some(q_end) = trimmed[abs_q_start..].find('"') {
                    let str_content = &trimmed[abs_q_start..abs_q_start + q_end];
                    let str_trimmed = str_content.trim();
                    if (str_trimmed.starts_with("query")
                        || str_trimmed.starts_with("mutation")
                        || str_trimmed.starts_with("subscription"))
                        && str_trimmed.contains('{')
                    {
                        queries.push(ExtractedQuery {
                            file: path.to_path_buf(),
                            line: line_idx + 1,
                            query: str_content.to_string(),
                        });
                    }
                    pos = abs_q_start + q_end + 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    queries
}

/// Parse the schema and build lookup maps for validation.
struct SchemaInfo {
    /// Fields available on the Query type: field_name -> set of argument names
    query_fields: HashMap<String, HashSet<String>>,
    /// Fields available on the Mutation type: field_name -> set of argument names
    mutation_fields: HashMap<String, HashSet<String>>,
    /// All named types in the schema (objects, scalars, enums, input objects, etc.)
    type_names: HashSet<String>,
}

fn load_schema(path: &Path) -> SchemaInfo {
    let schema_str = std::fs::read_to_string(path).expect("Failed to read schema file");
    let schema = cynic_parser::parse_type_system_document(&schema_str)
        .expect("Schema file is not valid GraphQL SDL");

    let mut query_fields: HashMap<String, HashSet<String>> = HashMap::new();
    let mut mutation_fields: HashMap<String, HashSet<String>> = HashMap::new();
    let mut type_names: HashSet<String> = HashSet::new();

    // Built-in scalar types are always available
    for builtin in ["String", "Int", "Float", "Boolean", "ID"] {
        type_names.insert(builtin.to_string());
    }

    use cynic_parser::type_system::{Definition, TypeDefinition};

    for def in schema.definitions() {
        match def {
            Definition::Type(ty) | Definition::TypeExtension(ty) => {
                type_names.insert(ty.name().to_string());

                match ty {
                    TypeDefinition::Object(obj) if obj.name() == "Query" => {
                        for field in obj.fields() {
                            let args: HashSet<String> =
                                field.arguments().map(|a| a.name().to_string()).collect();
                            query_fields.insert(field.name().to_string(), args);
                        }
                    }
                    TypeDefinition::Object(obj) if obj.name() == "Mutation" => {
                        for field in obj.fields() {
                            let args: HashSet<String> =
                                field.arguments().map(|a| a.name().to_string()).collect();
                            mutation_fields.insert(field.name().to_string(), args);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    SchemaInfo {
        query_fields,
        mutation_fields,
        type_names,
    }
}

/// Validate a parsed query against the schema.
/// Returns a list of validation errors.
fn validate_against_schema(
    doc: &cynic_parser::ExecutableDocument,
    schema: &SchemaInfo,
) -> Vec<String> {
    let mut errors = Vec::new();

    for op in doc.operations() {
        let op_type = op.operation_type();
        let root_fields = match op_type {
            cynic_parser::common::OperationType::Query => &schema.query_fields,
            cynic_parser::common::OperationType::Mutation => &schema.mutation_fields,
            cynic_parser::common::OperationType::Subscription => {
                // We don't use subscriptions, skip validation
                continue;
            }
        };

        let op_type_name = match op_type {
            cynic_parser::common::OperationType::Query => "Query",
            cynic_parser::common::OperationType::Mutation => "Mutation",
            cynic_parser::common::OperationType::Subscription => "Subscription",
        };

        // Check that top-level field selections exist in the root type
        for selection in op.selection_set() {
            if let cynic_parser::executable::Selection::Field(field) = selection {
                let field_name = field.name();
                // __typename and other introspection fields are always valid
                if field_name.starts_with("__") {
                    continue;
                }
                if !root_fields.contains_key(field_name) {
                    errors.push(format!(
                        "field `{field_name}` does not exist on type `{op_type_name}`"
                    ));
                }
            }
        }

        // Check that variable types reference real schema types
        for var in op.variable_definitions() {
            let type_name = var.ty().name();
            if !schema.type_names.contains(type_name) {
                errors.push(format!(
                    "variable `${}` references unknown type `{}`",
                    var.name(),
                    type_name
                ));
            }
        }
    }

    errors
}

/// Collect all source files that may contain GraphQL queries.
fn source_files() -> Vec<PathBuf> {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();

    // src/commands/*.rs
    let commands_dir = project_root.join("src/commands");
    if commands_dir.exists() {
        for entry in std::fs::read_dir(&commands_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }

    // src/client/mod.rs
    let client_mod = project_root.join("src/client/mod.rs");
    if client_mod.exists() {
        files.push(client_mod);
    }

    files.sort();
    files
}

#[test]
fn all_graphql_queries_are_valid() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let schema_path = project_root.join("schemas/linear.graphql");
    let schema = load_schema(&schema_path);

    let files = source_files();
    let mut all_queries = Vec::new();
    for file in &files {
        all_queries.extend(extract_queries_from_file(file));
    }

    assert!(
        !all_queries.is_empty(),
        "No GraphQL queries found in source files -- extraction logic may be broken"
    );

    let mut syntax_errors: Vec<String> = Vec::new();
    let mut schema_errors: Vec<String> = Vec::new();
    let mut total_validated = 0;

    for eq in &all_queries {
        let rel_path = eq
            .file
            .strip_prefix(&project_root)
            .unwrap_or(&eq.file)
            .display();

        match cynic_parser::parse_executable_document(&eq.query) {
            Ok(doc) => {
                total_validated += 1;

                // Schema-level validation
                let errors = validate_against_schema(&doc, &schema);
                for err in errors {
                    schema_errors.push(format!("  {}:{} -- {}", rel_path, eq.line, err));
                }
            }
            Err(e) => {
                syntax_errors.push(format!(
                    "  {}:{} -- {}\n    query: {:?}",
                    rel_path,
                    eq.line,
                    e,
                    truncate(&eq.query, 120)
                ));
            }
        }
    }

    eprintln!(
        "\n  Validated {} GraphQL queries across {} files",
        total_validated,
        files.len()
    );

    let mut report = String::new();

    if !syntax_errors.is_empty() {
        report.push_str(&format!(
            "\n{} GraphQL syntax error(s):\n{}\n",
            syntax_errors.len(),
            syntax_errors.join("\n")
        ));
    }

    if !schema_errors.is_empty() {
        report.push_str(&format!(
            "\n{} GraphQL schema validation error(s):\n{}\n",
            schema_errors.len(),
            schema_errors.join("\n")
        ));
    }

    assert!(report.is_empty(), "{report}");
}

/// Truncate a string for display in error messages.
fn truncate(s: &str, max: usize) -> String {
    let flat: String = s.chars().filter(|c| !c.is_ascii_control()).collect();
    if flat.len() > max {
        format!("{}...", &flat[..max])
    } else {
        flat
    }
}

#[test]
fn schema_parses_successfully() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let schema_path = project_root.join("schemas/linear.graphql");
    let schema_str = std::fs::read_to_string(&schema_path).expect("Failed to read schema");

    let schema = cynic_parser::parse_type_system_document(&schema_str)
        .expect("Schema should be valid GraphQL SDL");

    // Sanity check: the schema should have Query and Mutation types
    use cynic_parser::type_system::{Definition, TypeDefinition};

    let type_names: Vec<&str> = schema
        .definitions()
        .filter_map(|def| match def {
            Definition::Type(TypeDefinition::Object(obj)) => Some(obj.name()),
            _ => None,
        })
        .collect();

    assert!(
        type_names.contains(&"Query"),
        "Schema should contain a Query type"
    );
    assert!(
        type_names.contains(&"Mutation"),
        "Schema should contain a Mutation type"
    );

    eprintln!("\n  Schema has {} type definitions", type_names.len());
}

#[test]
fn query_extraction_finds_all_patterns() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Test extraction from a file we know has both raw and regular string queries
    let client_mod = project_root.join("src/client/mod.rs");
    let queries = extract_queries_from_file(&client_mod);

    // client/mod.rs has both r#"..."# and "..." style queries
    assert!(
        queries.len() >= 3,
        "Expected at least 3 queries from client/mod.rs, found {}",
        queries.len()
    );

    // Check that we found both regular and raw string queries
    let has_inline = queries.iter().any(|q| q.query.contains("teams"));
    let has_raw = queries.iter().any(|q| q.query.contains("workflowStates"));

    assert!(has_inline, "Should find inline string query (teams)");
    assert!(has_raw, "Should find raw string query (workflowStates)");
}
