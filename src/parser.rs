use anyhow::{ Context, Result };
use serde_json::Value;
use std::collections::{ BTreeMap, HashSet };
use std::fs;

use crate::models::{ ApiOperation, FieldData, Service, TypeDefinition };

/// Read and parse swagger JSON file
pub fn read_swagger_file(path: &str) -> Result<Value> {
    let content = fs
        ::read_to_string(path)
        .with_context(|| format!("Failed to read swagger file: {}", path))?;
    serde_json::from_str(&content).context("Invalid JSON in swagger file")
}

/// Parse swagger JSON into organized services
pub fn parse_swagger(
    swagger: &Value,
    tag_filters: Option<HashSet<String>>
) -> Result<Vec<Service>> {
    let paths = swagger
        .get("paths")
        .and_then(|p| p.as_object())
        .context("No 'paths' found in swagger file")?;

    let schemas_root = find_schemas(&swagger);

    let mut service_map: BTreeMap<String, Service> = BTreeMap::new();

    // Group operations by tag
    for (path, path_item) in paths.iter() {
        if let Some(obj) = path_item.as_object() {
            for (method, operation) in obj.iter() {
                if !is_valid_http_method(method) {
                    continue;
                }

                let tag_name = extract_tag(operation).unwrap_or_else(|| "default".to_string());
                let tag_normalized = normalize_tag(&tag_name);

                // Apply tag filter if provided
                // if let Some(ref filters) = tag_filters {
                // 现代写法
                if let Some(filters) = &tag_filters {
                    if !filters.contains(&tag_normalized) {
                        continue;
                    }
                }

                let api_op = parse_operation(
                    operation,
                    path,
                    method,
                    get_service(&mut service_map, &tag_normalized),
                    &schemas_root
                )?;

                get_service(&mut service_map, &tag_normalized).operations.push(api_op);
            }
        }
    }

    // Extract type definitions from schemas
    if let Some(schemas) = schemas_root {
        if let Some(schema_obj) = schemas.as_object() {
            for (name, schema) in schema_obj.iter() {
                for service in service_map.values_mut() {
                    if should_include_type(name, &service.operations) {
                        if let Ok(type_def) = extract_type_definition(name, schema) {
                            service.type_definitions.insert(name.clone(), type_def);
                        }
                    }
                }
            }
        }
    }

    Ok(service_map.into_values().collect())
}

fn get_service<'a>(service_map: &'a mut BTreeMap<String, Service>, name: &str) -> &'a mut Service {
    service_map.entry(name.to_string()).or_insert_with(|| Service {
        name: name.to_string(),
        operations: Vec::new(),
        type_definitions: BTreeMap::new(),
    })
}
/// Find schemas in either Swagger 2.0 or OpenAPI 3.0 format
fn find_schemas(swagger: &Value) -> Option<Value> {
    if let Some(defs) = swagger.get("definitions") {
        Some(defs.clone())
    } else if let Some(components) = swagger.get("components") {
        components.get("schemas").cloned()
    } else {
        None
    }
}

/// Check if method is a valid HTTP method
fn is_valid_http_method(method: &str) -> bool {
    matches!(method, "get" | "post" | "put" | "delete" | "patch" | "head" | "options")
}

/// Extract tag name from operation
fn extract_tag(operation: &Value) -> Option<String> {
    operation
        .get("tags")
        .and_then(|t| t.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Normalize tag name to lowercase
fn normalize_tag(tag: &str) -> String {
    tag.trim().to_lowercase()
}

/// Check if type name should be included in service
fn should_include_type(type_name: &str, operations: &[ApiOperation]) -> bool {
    operations
        .iter()
        .any(|op| { op.request_type.contains(type_name) || op.response_type.contains(type_name) })
}

/// Parse a single API operation
fn parse_operation(
    operation: &Value,
    path: &str,
    method: &str,
    service: &mut Service,
    _schemas: &Option<Value>
) -> Result<ApiOperation> {
    let function_name = extract_function_name(operation, method, path);
    let (request_type, response_type) = extract_types(operation, service);
    let operation_id = operation
        .get("operationId")
        .and_then(|v| v.as_str())
        .map(String::from);

    Ok(ApiOperation {
        path: path.to_string(),
        method: method.to_uppercase(),
        function_name,
        request_type,
        response_type,
        operation_id,
    })
}

/// Extract or generate function name
fn extract_function_name(operation: &Value, method: &str, path: &str) -> String {
    // First try operationId
    if let Some(opid) = operation.get("operationId").and_then(|v| v.as_str()) {
        return opid.to_string();
    }

    // Fallback: generate from method and path
    generate_function_name(method, path)
}

static METHOD_OP_MAP: [(&str, &str); 7] = [
    ("get", "Get"),
    ("post", "Create"),
    ("put", "Update"),
    ("delete", "Delete"),
    ("patch", "Patch"),
    ("head", "Head"),
    ("options", "Options"),
];

/// Generate camelCase function name from HTTP method and path
fn generate_function_name(method: &str, path: &str) -> String {
    let method_lower = method.to_lowercase();
    let path_clean = path.replace('/', " ").replace('{', " by ").replace('}', "");

    let parts: Vec<_> = path_clean
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .collect();

    let mut result = METHOD_OP_MAP.iter()
        .find(|&&(m, _)| m == method_lower)
        .map(|&(_, op)| op)
        .unwrap_or("")
        .to_string();

    for part in parts {
        let capitalized = capitalize_first(part);
        result.push_str(&capitalized);
    }

    result
}

/// Capitalize first character of string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let mut result = String::new();
            result.push_str(&first.to_uppercase().collect::<String>());
            result.push_str(chars.as_str());
            result
        }
    }
}

/// Extract request and response types from operation
fn extract_types(operation: &Value, service: &mut Service) -> (String, String) {
    let mut request_type = String::from("any");
    let mut response_type = String::from("any");

    // Extract request type from parameters（2.0） or requestBody（3.0）
    if let Some(params) = operation.get("parameters").and_then(|v| v.as_array()) {
        for param in params {
            if let Some(schema) = param.get("schema") {
                request_type = extract_type_name_from_schema(schema);
                if !request_type.is_empty() && request_type != "any" {
                    break;
                }
            }
        }
    }

    if request_type == "any" {
        if let Some(rb) = operation.get("requestBody") {
            if let Some(content) = rb.get("content") {
                if let Some(appjson) = content.get("application/json") {
                    if let Some(schema) = appjson.get("schema") {
                        request_type = extract_type_name_from_schema(schema);
                    }
                }
            }
        }
    }

    //解析parameters的每一个param, 构建新的对象
    if request_type == "any" {
        let params = operation
            .get("parameters")
            .and_then(|v| v.as_array())
            .map(|v| v.to_owned())
            .unwrap_or(vec![]);
        if !params.is_empty() {
            let type_name = format!("{}Request", capitalize_first(&service.name));
            let mut custom_type = TypeDefinition {
                name: type_name.clone(),
                fields: BTreeMap::new(),
                description: None,
            };
            for param in params {
                if let Some(field_name) = param.get("name").and_then(|v| v.as_str()) {
                    if let Some(field_type) = param.get("type").and_then(|v| v.as_str()) {
                        let js_type = match field_type {
                            "string" => "string",
                            "integer" | "number" | "float" | "double" => "number",
                            "boolean" => "boolean",
                            _ => "any",
                        };
                        custom_type.fields.insert(field_name.to_string(), FieldData {
                            field_type: js_type.to_string(),
                            optional: param
                                .get("required")
                                .and_then(|v| v.as_bool().map(|b| !b))
                                .unwrap_or(true),
                            description: None,
                        });
                    }
                }
            }
            request_type = type_name.clone();
            service.type_definitions.insert(type_name.clone(), custom_type);
        }
    }

    // Extract response type
    if let Some(responses) = operation.get("responses").and_then(|v| v.as_object()) {
        let response_schema = responses
            .get("200")
            .or_else(|| responses.get("201"))
            .or_else(|| responses.get("default"))
            .or_else(|| responses.values().next());

        if let Some(resp) = response_schema {
            if let Some(schema) = resp.get("schema") {
                response_type = extract_type_name_from_schema(schema);
            } else if let Some(content) = resp.get("content") {
                if let Some(appjson) = content.get("application/json") {
                    if let Some(schema) = appjson.get("schema") {
                        response_type = extract_type_name_from_schema(schema);
                    }
                }
            }
        }
    }

    (
        if request_type.is_empty() { "any".to_string() } else { request_type },
        if response_type.is_empty() { "any".to_string() } else { response_type },
    )
}

/// Extract type name from schema (handles $ref)
fn extract_type_name_from_schema(schema: &Value) -> String {
    if let Some(ref_str) = schema.get("$ref").and_then(|v| v.as_str()) {
        return ref_str.split('/').last().unwrap_or("any").to_string();
    }

    if let Some(type_str) = schema.get("type").and_then(|v| v.as_str()) {
        match type_str {
            "string" => "string".to_string(),
            "integer" | "number" | "float" | "double" => "number".to_string(),
            "boolean" => "boolean".to_string(),
            "array" => {
                if let Some(items) = schema.get("items") {
                    format!("{}[]", extract_type_name_from_schema(items))
                } else {
                    "any[]".to_string()
                }
            }
            _ => "any".to_string(),
        }
    } else {
        "any".to_string()
    }
}

/// Extract type definition from schema
fn extract_type_definition(name: &str, schema: &Value) -> Result<TypeDefinition> {
    let mut fields = BTreeMap::new();

    if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
        let required_fields = schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|v| v.to_owned())
            .unwrap_or(vec![]);
        let required_fields_set: HashSet<String> = required_fields
            .iter()
            .map(|v| v.as_str())
            .map(|v| v.expect("required field is not a string").to_string())
            .collect();
        for (field_name, field_schema) in props.iter() {
            let field_type = extract_type_name_from_schema(field_schema);
            fields.insert(field_name.clone(), FieldData {
                field_type,
                optional: !required_fields_set.contains(field_name.as_str()),
                description: None,
            });
        }
    }

    let description = schema
        .get("description")
        .and_then(|v| v.as_str())
        .map(String::from);

    Ok(TypeDefinition {
        name: name.to_string(),
        fields,
        description,
    })
}
