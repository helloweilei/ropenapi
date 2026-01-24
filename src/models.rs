use std::collections::BTreeMap;

/// Represents a single API operation (GET, POST, etc.)
#[derive(Debug, Clone)]
pub struct ApiOperation {
    pub path: String,
    pub method: String,
    pub function_name: String,
    pub request_type: String,
    pub response_type: String,
    #[allow(dead_code)]
    pub operation_id: Option<String>,
}

/// Represents a service group (e.g., User, Order)
#[derive(Debug, Clone)]
pub struct Service {
    pub name: String,
    pub operations: Vec<ApiOperation>,
    pub type_definitions: BTreeMap<String, TypeDefinition>,
}

/// Represents a TypeScript type definition
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub name: String,
    pub fields: BTreeMap<String, String>,
    #[allow(dead_code)]
    pub description: Option<String>,
}

impl TypeDefinition {
    pub fn to_typescript(&self) -> String {
        if self.fields.is_empty() {
            format!("export type {} = any;", self.name)
        } else {
            let mut body = String::from("{\n");
            for (field_name, field_type) in &self.fields {
                body.push_str(&format!("  {}: {};\n", field_name, field_type));
            }
            body.push('}');
            format!("export type {} = {}", self.name, body)
        }
    }
}

impl ApiOperation {
    pub fn to_typescript_function(&self) -> String {
        let arg_name = match self.method.as_str() {
            "GET" | "DELETE" => "params",
            _ => "data",
        };

        let req_type = if self.request_type.is_empty() || self.request_type == "any" {
            "any".to_string()
        } else {
            format!("Types.{}", self.request_type)
        };

        let resp_type = if self.response_type.is_empty() || self.response_type == "any" {
            "any".to_string()
        } else {
            format!("Types.{}", self.response_type)
        };

        let url = self.path.trim_start_matches('/');

        format!(
            "export const {} = async ({}: {}): Promise<IResponse<{}>> => {{\n  return request<{}, {}>( {{\n    url: '{}',\n    {}: {},\n    method: '{}',\n  }} );\n}};",
            self.function_name,
            arg_name,
            req_type,
            resp_type,
            req_type,
            resp_type,
            url,
            arg_name,
            arg_name,
            self.method
        )
    }
}
