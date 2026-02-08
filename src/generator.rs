use anyhow::{ Context, Result };
use std::fs::{ self, OpenOptions };
use std::io::Write;
use std::path::Path;

use crate::models::Service;
use crate::cli::Args;
use crate::models::TypeDefinition;

/// Write all services to disk
pub fn write_services(out_dir: &Path, services: &[Service], args: &Args) -> Result<()> {
    let services_dir = out_dir.join(args.project_name.as_ref().unwrap_or(&String::from("")));
    ensure_dir(&services_dir)?;

    for service in services {
        write_service(
            &services_dir,
            service,
            args.request_lib_path
                .as_ref()
                .unwrap_or(&String::from("import request from \'@/services/http\';")),
            args.api_prefix.as_ref().unwrap_or(&String::from(""))
        )?;
    }

    Ok(())
}

/// Write a single service (both api file and types file)
fn write_service(
    services_root: &Path,
    service: &Service,
    request_lib: &str,
    api_prefix: &str
) -> Result<()> {
    let service_name = if service.name.to_lowercase().ends_with("Controller") {
        service.name.clone()
    } else {
        format!("{}Controller", capitalize(&service.name))
    };

    let file_path = services_root.join(format!("{}.ts", service_name));

    write_service_to_file(&file_path, service, request_lib, api_prefix)?;

    println!("  ✓ Generated {}/{}", service.name, file_path.file_name().unwrap().to_string_lossy());

    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn write_service_to_file(
    path: &Path,
    service: &Service,
    request_lib: &str,
    api_prefix: &str
) -> Result<()> {
    write_api_file_with_request_lib(path, service, request_lib, api_prefix)?;
    write_types_file(path, service)?;

    Ok(())
}

/// Write API service file
fn write_api_file_with_request_lib(
    path: &Path,
    service: &Service,
    request_lib: &str,
    api_prefix: &str
) -> Result<()> {
    let mut content = String::new();

    // Add header with imports
    content.push_str("// @ts-expect-error\n");
    content.push_str(request_lib);
    content.push_str("\n\n");
    // content.push_str("import * as Types from './types';\n");
    // content.push_str("import type { IResponse } from '@/types';\n\n");

    // Add operations with proper spacing
    for (idx, operation) in service.operations.iter().enumerate() {
        content.push_str(&operation.to_typescript_function(api_prefix));
        if idx < service.operations.len() - 1 {
            content.push_str("\n\n");
        }
    }

    content.push('\n');

    fs
        ::write(path, &content)
        .with_context(|| format!("Failed to write API file: {}", path.display()))?;

    Ok(())
}

/// Write types definition file
fn write_types_file(path: &Path, service: &Service) -> Result<()> {
    let mut content = String::new();

    content.push_str("/**\n");
    content.push_str(&format!(" * Type definitions for {} service\n", service.name));
    content.push_str(" */\n\n");

    let type_defs = service.type_definitions.values().collect::<Vec<_>>();

    for type_def in type_defs.clone() {
        content.push_str(&type_def.to_typescript());
        content.push_str("\n\n");
    }

    // Add placeholder types for operations if not in definitions
    for operation in &service.operations {
        if
            !type_already_exists(type_defs.clone(), &operation.request_type) &&
            operation.request_type != "any"
        {
            content.push_str(&format!("export type {} = any;\n\n", operation.request_type));
        }
        if
            !type_already_exists(type_defs.clone(), &operation.response_type) &&
            operation.response_type != "any"
        {
            content.push_str(&format!("export type {} = any;\n\n", operation.response_type));
        }
    }

    // Trim trailing whitespace
    let content = content.trim_end().to_string() + "\n";

    if path.exists() {
        OpenOptions::new().append(true).open(path)?.write_all(content.as_bytes())?;
    } else {
        fs
            ::write(path, &content)
            .with_context(|| format!("Failed to write types file: {}", path.display()))?;
    }

    Ok(())
}

/// Check if type is already defined in content
fn type_already_exists<'a>(type_defs: Vec<&TypeDefinition>, type_name: &str) -> bool {
    type_defs.iter().any(|type_def| type_def.name == type_name)
}

/// Ensure directory exists
fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        fs
            ::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}
